//! Defines a simple print queue abstraction.

use crate::cli::Format;
use anyhow::Error;
use anyhow::Result;
use serde_json::json;
use serde_json::Value as JsonValue;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::future::Future;
use std::panic;
use std::path::Path;
use std::result::Result as StdResult;
use std::{fmt::Display, io::Write};
use tokio::io::stderr;
use tokio::io::stdout;
use tokio::io::AsyncWriteExt as _;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinError;
use tracing::debug;
use url::Url;

/// A handle to assist in interacting with the printer.
pub struct Printer {
    /// The transmitter to send message to the task.
    tx: Sender<PrinterCmd>,

    /// The actual future to be awaited.
    task: Box<dyn Future<Output = StdResult<(), JoinError>> + Unpin>,
}

impl Printer {
    /// Launch the print queue task, give back sender and future for it.
    pub fn launch(buffer_size: usize) -> Printer {
        let (tx, mut rx) = mpsc::channel::<PrinterCmd>(buffer_size);

        let printer = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                debug!(?msg);

                match msg {
                    // Closing the stream ensures it still drains if there are messages in flight.
                    PrinterCmd::End => rx.close(),
                    PrinterCmd::Message(msg) => {
                        let format = msg.format();
                        let msg_clone = msg.clone();

                        if let Err(err) = msg.print().await {
                            // Fallback to only sync printing if the async printing failed.
                            let err_msg = Msg::error(err, format);

                            if let Err(err) = err_msg.sync_print() {
                                panic!("failed to print sync error message: '{}'", err);
                            }

                            if let Err(err) = msg_clone.sync_print() {
                                panic!("failed to print sync message: '{}'", err);
                            }
                        }
                    }
                }
            }
        });

        Printer {
            tx,
            task: Box::new(printer),
        }
    }

    /// Send a message to the print task.
    pub async fn send(&self, cmd: PrinterCmd) {
        self.tx
            .send(cmd)
            .await
            .expect("print task is awaited and should still be receiving")
    }

    /// Wait on the underlying task.
    ///
    /// This function waits, and then either returns normally or panics.
    pub async fn join(self) {
        if let Err(error) = self.task.await {
            // If the print task panicked, the whole task should panic.
            if error.is_panic() {
                panic::resume_unwind(error.into_panic())
            }

            if error.is_cancelled() {
                panic!("the printer task was cancelled unexpectedly");
            }
        }
    }

    /// Get a reference to the task transmitter.
    pub fn tx(&self) -> &Sender<PrinterCmd> {
        &self.tx
    }
}

/// A print queue message, either an actual message or a signals to end printing.
#[derive(Debug, Clone)]
pub enum PrinterCmd {
    /// Shut down the printer task.
    End,

    /// Print the following message.
    Message(Msg),
}

impl PrinterCmd {
    /// Construct a new ID printer command.
    pub fn id(path: &Path, url: &Url, format: Format) -> Self {
        PrinterCmd::Message(Msg::id(path, url, format))
    }

    /// Construct a new find printer command.
    pub fn find(path: &Path, url: &Url, format: Format) -> Self {
        PrinterCmd::Message(Msg::find(path, url, format))
    }

    /// Construct a new error printer command.
    pub fn error<E: Into<Error>>(error: E, format: Format) -> PrinterCmd {
        PrinterCmd::Message(Msg::error(error, format))
    }

    pub fn root_dir(dir: Option<&Path>, format: Format) -> PrinterCmd {
        PrinterCmd::Message(Msg::root_dir(dir, format))
    }
}

/// An individual message to be printed.
#[derive(Debug, Clone)]
pub struct Msg {
    /// The message content.
    content: Content,

    /// The status associated with the message.
    status: Status,
}

impl Msg {
    /// Construct a new ID message.
    pub fn id(path: &Path, url: &Url, format: Format) -> Self {
        let status = Status::Success;
        let path = path.display().to_string();
        let url = url.to_string();

        match format {
            Format::Plain => Msg::plain(status, &format!("{} => {}", path, url)),
            Format::Short => Msg::plain(status, &url.to_string()),
            Format::Json => Msg::json(status, json!({ "path": path, "id": url })),
        }
    }

    /// Construct a new find message.
    pub fn find(path: &Path, url: &Url, format: Format) -> Self {
        let status = Status::Success;
        let path = path.display().to_string();
        let url = url.to_string();

        match format {
            Format::Plain => Msg::plain(status, &format!("{} => {}", url, path)),
            Format::Short => Msg::plain(status, &path.to_string()),
            Format::Json => Msg::json(status, json!({ "path": path, "id": url })),
        }
    }

    pub fn root_dir(dir: Option<&Path>, format: Format) -> Self {
        let status = Status::Success;
        let dir = match dir {
            Some(path) => path.display().to_string(),
            None => String::from("no OmniBOR root directory provided"),
        };

        match format {
            Format::Plain => Msg::plain(status, &format!("root_dir: {}", dir)),
            Format::Short => Msg::plain(status, &dir.to_string()),
            Format::Json => Msg::json(status, json!({ "root_dir": dir })),
        }
    }

    /// Construct a new error message.
    pub fn error<E: Into<Error>>(error: E, format: Format) -> Msg {
        fn _error(error: Error, format: Format) -> Msg {
            let status = Status::Error;

            match format {
                Format::Plain | Format::Short => Msg::plain(status, &format!("error: {}", error)),
                Format::Json => Msg::json(status, json!({"error": error.to_string()})),
            }
        }

        _error(error.into(), format)
    }

    /// Construct a new plain Msg.
    fn plain(status: Status, s: &str) -> Self {
        Msg {
            content: Content::Plain(s.to_string()),
            status,
        }
    }

    /// Construct a new JSON Msg.
    fn json(status: Status, j: JsonValue) -> Self {
        Msg {
            content: Content::Json(j),
            status,
        }
    }

    /// Get the format of the message.
    fn format(&self) -> Format {
        match self.content {
            Content::Json(_) => Format::Json,
            Content::Plain(_) => Format::Plain,
        }
    }

    /// Print the Msg to the appropriate sink.
    async fn print(self) -> Result<()> {
        let to_output = self.content.to_string();
        self.write(to_output.as_bytes()).await?;
        Ok(())
    }

    /// Print the contents of the message synchronously.
    fn sync_print(self) -> Result<()> {
        let to_output = self.content.to_string();
        let bytes = to_output.as_bytes();

        match self.status {
            Status::Success => std::io::stdout().write_all(bytes)?,
            Status::Error => std::io::stderr().write_all(bytes)?,
        }

        Ok(())
    }

    /// Write bytes to the correct sink.
    async fn write(&self, bytes: &[u8]) -> Result<()> {
        match self.status {
            Status::Success => stdout().write_all(bytes).await?,
            Status::Error => stderr().write_all(bytes).await?,
        }

        Ok(())
    }
}

/// The actual content of a message.
#[derive(Debug, Clone)]
pub enum Content {
    Json(JsonValue),
    Plain(String),
}

impl Display for Content {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Content::Plain(s) => writeln!(f, "{}", s),
            Content::Json(j) => writeln!(f, "{}", j),
        }
    }
}

/// Whether the message is a success or error.
#[derive(Debug, Clone, Copy)]
pub enum Status {
    Success,
    Error,
}
