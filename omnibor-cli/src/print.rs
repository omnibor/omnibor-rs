//! Defines a simple print queue abstraction.

use crate::cli::Format;
use anyhow::Error;
use anyhow::Result;
use serde_json::json;
use serde_json::Value as JsonValue;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::future::Future;
use std::panic;
use std::path::Path;
use std::result::Result as StdResult;
use tokio::io::stderr;
use tokio::io::stdout;
use tokio::io::AsyncWriteExt as _;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinError;
use url::Url;

const DEFAULT_BUFFER_SIZE: usize = 100;

/// A handle to assist in interacting with the printer.
pub struct Printer {
    /// The transmitter to send message to the task.
    tx: Sender<PrinterCmd>,

    /// The actual future to be awaited.
    task: Box<dyn Future<Output = StdResult<(), JoinError>> + Unpin>,
}

impl Printer {
    /// Launch the print queue task, give back sender and future for it.
    pub fn launch(buffer_size: Option<usize>) -> Printer {
        let (tx, mut rx) = mpsc::channel::<PrinterCmd>(buffer_size.unwrap_or(DEFAULT_BUFFER_SIZE));

        let printer = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    PrinterCmd::End => break,
                    PrinterCmd::Message(msg) => {
                        // TODO(alilleybrinker): Handle this error.
                        let _ = msg.print().await;
                    }
                }
            }
            rx.close();
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

    /// Construct a new error message.
    pub fn error<E: Into<Error>>(error: E, format: Format) -> Msg {
        fn _error(error: Error, format: Format) -> Msg {
            let status = Status::Error;

            match format {
                Format::Plain | Format::Short => {
                    Msg::plain(status, &format!("error: {}", error))
                }
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

    /// Print the Msg to the appropriate sink.
    async fn print(self) -> Result<()> {
        let to_output = self.content.to_string();
        self.write(to_output.as_bytes()).await?;
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
