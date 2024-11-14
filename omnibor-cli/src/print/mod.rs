//! Defines a simple print queue abstraction.

pub mod error;
pub mod find_file;
pub mod id_file;
pub mod root_dir;

use crate::{
    cli::Format,
    error::{Error, Result},
};
use dyn_clone::{clone_box, DynClone};
use error::ErrorMsg;
use serde_json::Value as JsonValue;
use std::{
    fmt::Debug,
    future::Future,
    io::Write,
    ops::{Deref, Not},
    panic,
    result::Result as StdResult,
};
use tokio::{
    sync::mpsc::{self, Sender},
    task::JoinError,
};
use tracing::{debug, error};

/// A handle to assist in interacting with the printer.
pub struct Printer {
    /// The transmitter to send message to the task.
    tx: PrintSender,

    /// The actual future to be awaited.
    task: Box<dyn Future<Output = StdResult<(), JoinError>> + Unpin>,
}

impl Printer {
    /// Launch the print queue task, give back sender and future for it.
    pub fn launch(buffer_size: usize) -> Printer {
        let (tx, mut rx) = mpsc::channel::<PrinterCmd>(buffer_size);

        let printer = tokio::task::spawn_blocking(move || {
            while let Some(msg) = rx.blocking_recv() {
                debug!(msg = ?msg);

                match msg {
                    // Closing the stream ensures it still drains if there are messages in flight.
                    PrinterCmd::End => rx.close(),
                    PrinterCmd::Message { output, format } => {
                        let status = output.status();
                        let output = output.format(format);

                        if let Err(error) = sync_print(status, output.clone()) {
                            let err_output = ErrorMsg::new(error).format(format);

                            if let Err(err) = sync_print(Status::Error, err_output) {
                                error!(msg = "failed to print sync error message", error = %err);
                            }
                        }
                    }
                }
            }
        });

        Printer {
            tx: PrintSender(tx),
            task: Box::new(printer),
        }
    }

    /// Send a message to the print task.
    pub async fn send(&self, cmd: PrinterCmd) {
        if let Err(e) = self.tx.send(cmd.clone()).await {
            error!(msg = "failed to send printer cmd", cmd = ?cmd, error = %e);
        }
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
                error!(msg = "the printer task was cancelled unexpectedly");
            }
        }
    }

    /// Get a reference to the task transmitter.
    pub fn tx(&self) -> &PrintSender {
        &self.tx
    }
}

pub struct PrintSender(Sender<PrinterCmd>);

impl PrintSender {
    pub async fn send(&self, value: PrinterCmd) -> Result<()> {
        self.0
            .send(value)
            .await
            .map_err(|_| Error::PrintChannelClose)
    }
}

impl Clone for PrintSender {
    fn clone(&self) -> Self {
        PrintSender(self.0.clone())
    }
}

/// A print queue message, either an actual message or a signals to end printing.
#[derive(Debug, Clone)]
pub enum PrinterCmd {
    /// Shut down the printer task.
    End,

    /// Print the following message.
    Message { output: Msg, format: Format },
}

impl PrinterCmd {
    /// Construct a new ID printer command.
    pub fn msg<C: CommandOutput>(output: C, format: Format) -> Self {
        PrinterCmd::Message {
            output: Msg::new(output),
            format,
        }
    }
}

#[derive(Debug)]
pub struct Msg(Box<dyn CommandOutput>);

impl Clone for Msg {
    fn clone(&self) -> Self {
        Msg(clone_box(self.0.deref()))
    }
}

impl Deref for Msg {
    type Target = Box<dyn CommandOutput>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Msg {
    fn new<C: CommandOutput>(output: C) -> Self {
        Msg(Box::new(output))
    }
}

/// Trait representing the different possible outputs that can arise.
pub trait CommandOutput: Debug + DynClone + Send + 'static {
    fn plain_output(&self) -> String;
    fn short_output(&self) -> String;
    fn json_output(&self) -> JsonValue;
    fn status(&self) -> Status;

    fn format(&self, format: Format) -> String {
        let mut output = match format {
            Format::Plain => self.plain_output(),
            // SAFETY: serde_json::Value can always be converted to a string.
            Format::Json => serde_json::to_string(&self.json_output()).unwrap(),
            Format::Short => self.short_output(),
        };

        if output.ends_with('\n').not() {
            output.push('\n');
        }

        output
    }
}

/// Print the contents of the message synchronously.
fn sync_print(status: Status, output: String) -> Result<()> {
    let bytes = output.as_bytes();

    match status {
        Status::Success => std::io::stdout()
            .write_all(bytes)
            .map_err(Error::StdoutWriteFailed)?,
        Status::Error => std::io::stderr()
            .write_all(bytes)
            .map_err(Error::StderrWriteFailed)?,
    }

    Ok(())
}

/// Whether the message is a success or error.
#[derive(Debug, Clone, Copy)]
pub enum Status {
    Success,
    Error,
}
