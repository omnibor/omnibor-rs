use anyhow::{bail, Error, Result};
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::iter::DoubleEndedIterator;
use std::result::Result as StdResult;

/// A mutable-reference [`Step`]` trait object.
pub type DynStep<'step> = &'step mut dyn Step;

/// Run a pipeline of steps in order, rolling back if needed.
///
/// The type signature here is a little funky, but it just means that
/// it takes as a parameter something which can be turned into an owning
/// iterator over mutable references to Step trait objects.
///
/// This lets the user call it with just a plain array of trait objects,
/// also assisted by the `step!` macro.
pub fn run<'step, I, It>(steps: I) -> Result<()>
where
    It: DoubleEndedIterator<Item = DynStep<'step>>,
    I: IntoIterator<Item = DynStep<'step>, IntoIter = It>,
{
    fn inner<'step>(mut steps: impl DoubleEndedIterator<Item = DynStep<'step>>) -> Result<()> {
        while let Some(step) = steps.next() {
            if let Err(forward) = forward(step) {
                while let Some(reverse_step) = steps.next_back() {
                    if let Err(backward) = backward(reverse_step) {
                        bail!(PipelineError::rollback(forward, backward));
                    }
                }

                bail!(PipelineError::forward(forward));
            }
        }

        Ok(())
    }

    inner(steps.into_iter())
}

#[macro_export]
macro_rules! step {
    ( $step:expr ) => {{
        &mut $step as &mut dyn Step
    }};
}

/// A pipeline step which mutates the environment and can be undone.
pub trait Step {
    /// The name of the step, to report to the user.
    ///
    /// # Note
    ///
    /// This should _always_ return a consistent name for the step,
    /// not based on any logic related to the arguments passed to the
    /// program.
    ///
    /// This is a method, not an associated function, to ensure that
    /// the [`Step`] trait is object-safe. The `pipeline::run` function
    /// runs steps through an iterator of `Step` trait objects, so this
    /// is a requirement of the design.
    fn name(&self) -> &'static str;

    /// Do the step.
    ///
    /// Steps are expected to clean up after themselves for the forward
    /// direction if they fail after partial completion. The `undo` is
    /// only for undoing a completely successful forward step if a later
    /// step fails.
    fn run(&mut self) -> Result<()>;

    /// Undo the step.
    ///
    /// This is run automatically by the pipelining system if there's
    /// a need to rollback the pipeline because a later step failed.
    ///
    /// This is to ensure that any pipeline of operations operates
    /// a single cohesive whole, either _all_ completing or _none_
    /// visibly completing by the end.
    ///
    /// Note that this trait does _not_ ensure graceful shutdown if
    /// you cancel an operation with a kill signal before the `undo`
    /// operation can complete.
    fn undo(&mut self) -> Result<()>;
}

/// Helper function to run a step forward and convert the error to [`StepError`]
fn forward(step: &mut dyn Step) -> StdResult<(), StepError> {
    log::info!("running step '{}'", step.name());

    step.run().map_err(|error| StepError {
        name: step.name(),
        error,
    })
}

/// Helper function to run a step backward and convert the error to [`StepError`]
fn backward(step: &mut dyn Step) -> StdResult<(), StepError> {
    log::info!("rolling back step '{}'", step.name());

    step.undo().map_err(|error| StepError {
        name: step.name(),
        error,
    })
}

/// An error from running a pipeline of steps.
#[derive(Debug)]
enum PipelineError {
    /// An error arose during forward execution.
    Forward {
        /// The error produced by the offending step.
        forward: StepError,
    },
    /// An error arose during forward execution and also during rollback.
    Rollback {
        /// The name of the forward step that errored.
        forward_name: &'static str,

        /// The name of the backward step that errored.
        backward_name: &'static str,

        /// A combination of the backward and forward error types.
        rollback: Error,
    },
}

impl PipelineError {
    /// Construct a forward error.
    fn forward(forward: StepError) -> Self {
        PipelineError::Forward { forward }
    }

    /// Construct a rollback error.
    fn rollback(forward: StepError, backward: StepError) -> Self {
        let forward_name = forward.name;
        let backward_name = backward.name;
        let rollback = Error::new(backward).context(forward);

        PipelineError::Rollback {
            forward_name,
            backward_name,
            rollback,
        }
    }
}

impl Display for PipelineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            PipelineError::Forward { forward } => {
                write!(f, "{}, but rollback was successful", forward)
            }
            PipelineError::Rollback {
                forward_name,
                backward_name,
                ..
            } => write!(
                f,
                "step '{}' failed and step '{}' failed to rollback",
                forward_name, backward_name
            ),
        }
    }
}

impl StdError for PipelineError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            PipelineError::Forward { forward } => Some(forward),
            PipelineError::Rollback { rollback, .. } => Some(rollback.as_ref()),
        }
    }
}

/// An error from a single pipeline step.
#[derive(Debug)]
struct StepError {
    /// The name of the step that errored.
    name: &'static str,

    /// The error the step produced.
    error: Error,
}

impl Display for StepError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "step '{}' failed", self.name)
    }
}

impl StdError for StepError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(self.error.as_ref())
    }
}
