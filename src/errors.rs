use miette::Diagnostic;
use thiserror::Error;

use crate::BoxedError;

/// This enum contains all the possible errors that could be returned
/// by [`handle_shutdown_requests()`](crate::Toplevel::handle_shutdown_requests).
#[derive(Error, Debug, Diagnostic)]
pub enum GracefulShutdownError {
    /// At least one subsystem caused an error.
    #[error("at least one subsystem returned an error")]
    SubsystemsFailed(#[related] Vec<SubsystemError>),
    /// The shutdown did not finish within the given timeout.
    #[error("shutdown timed out")]
    ShutdownTimeout(#[related] Vec<SubsystemError>),
}

impl GracefulShutdownError {
    #[doc(hidden)]
    pub fn into_related(self) -> Vec<SubsystemError> {
        match self {
            GracefulShutdownError::SubsystemsFailed(rel) => rel,
            GracefulShutdownError::ShutdownTimeout(rel) => rel,
        }
    }
}

/// This enum contains all the possible errors that a partial shutdown
/// could cause.
#[derive(Debug, Error, Diagnostic)]
pub enum PartialShutdownError {
    /// At least one subsystem caused an error.
    #[error("at least one subsystem returned an error")]
    SubsystemsFailed(#[related] Vec<SubsystemError>),
    /// The given nested subsystem does not seem to be a child of
    /// the parent subsystem.
    #[error("unable to find nested subsystem in given subsystem")]
    SubsystemNotFound,
    /// A partial shutdown can not be performed because the entire program
    /// is already shutting down.
    #[error("unable to perform partial shutdown, the program is already shutting down")]
    AlreadyShuttingDown,
}

/// This enum contains all the possible errors that a subsystem execution
/// could cause.
///
/// Every error carries the name of the subsystem as the first argument.
#[derive(Debug, Error, Diagnostic)]
pub enum SubsystemError {
    /// The subsystem returned an error value. Carries the actual error as the second argument.
    #[error("Error in subsystem '{0}'")]
    Failed(String, #[source] BoxedError),
    /// The subsystem was cancelled. Should only happen if the shutdown timeout is exceeded.
    #[error("Subsystem '{0}' was aborted")]
    Cancelled(String),
    /// The subsystem panicked.
    #[error("Subsystem '{0}' panicked")]
    Panicked(String),
}

impl SubsystemError {
    /// Retrieves the name of the subsystem that caused the error.
    ///
    /// # Returns
    ///
    /// The name of the subsystem
    pub fn name(&self) -> &str {
        match self {
            SubsystemError::Failed(name, _) => name,
            SubsystemError::Cancelled(name) => name,
            SubsystemError::Panicked(name) => name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn examine_report(report: miette::Report) {
        println!("{}", report);
        println!("{:?}", report);
        // Convert to std::error::Error
        let boxed_error: Box<dyn std::error::Error + Send + Sync> = report.into();
        println!("{}", boxed_error);
        println!("{:?}", boxed_error);
    }

    #[test]
    fn errors_can_be_converted_to_diagnostic() {
        examine_report(GracefulShutdownError::ShutdownTimeout(vec![]).into());
        examine_report(GracefulShutdownError::SubsystemsFailed(vec![]).into());
        examine_report(PartialShutdownError::AlreadyShuttingDown.into());
        examine_report(PartialShutdownError::SubsystemNotFound.into());
        examine_report(PartialShutdownError::SubsystemsFailed(vec![]).into());
        examine_report(SubsystemError::Cancelled("".into()).into());
        examine_report(SubsystemError::Panicked("".into()).into());
        examine_report(SubsystemError::Failed("".into(), "".into()).into());
    }

    #[test]
    fn convert_graceful_shutdown_error_into_related() {
        let related = || {
            vec![
                SubsystemError::Cancelled("a".into()),
                SubsystemError::Panicked("b".into()),
            ]
        };

        let matches_related = |data: Vec<SubsystemError>| {
            let mut iter = data.into_iter();

            let elem = iter.next().unwrap();
            assert_eq!(elem.name(), "a");
            assert!(matches!(elem, SubsystemError::Cancelled(_)));

            let elem = iter.next().unwrap();
            assert_eq!(elem.name(), "b");
            assert!(matches!(elem, SubsystemError::Panicked(_)));

            assert!(iter.next().is_none());
        };

        matches_related(GracefulShutdownError::ShutdownTimeout(related()).into_related());
        matches_related(GracefulShutdownError::SubsystemsFailed(related()).into_related());
    }
}
