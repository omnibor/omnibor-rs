use std::any::Any;
use std::cell::RefCell;
use std::error::Error as StdError;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::panic::catch_unwind;
use std::panic::UnwindSafe;

thread_local! {
    // The last error to have been reported by the FFI code.
    /// cbindgen:ignore
    static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);
}

/// Update the last error with a new error message.
#[inline]
pub(crate) fn set_error_msg(e: String) {
    LAST_ERROR.with(|last| {
        *last.borrow_mut() = Some(e);
    });
}

/// Get the last error message if there is one.
#[inline]
pub(crate) fn get_error_msg() -> Option<String> {
    LAST_ERROR.with(|last| last.borrow_mut().take())
}

/// Convenient panic-catching and reporting.
///
/// This wraps `std::panic::catch_unwind`, but enables you to write functions
/// which return `Result<T, Error>` and have those errors correctly
/// reported out.
pub(crate) fn catch_panic<T, F>(f: F) -> Option<T>
where
    F: FnOnce() -> Result<T, Error> + UnwindSafe,
{
    // The return type is Result<Result<T, Error>, AnyError>
    let result = catch_unwind(f);

    match result {
        Ok(Ok(value)) => Some(value),
        Ok(Err(err)) => {
            // We have our `Error` type.
            set_error_msg(err.to_string());
            None
        }
        Err(err) => {
            // We have a `Box<dyn Any + Send + 'static>`
            set_error_msg(ErrorMsg::from(err).to_string());
            None
        }
    }
}

#[derive(Debug)]
pub(crate) enum Error {
    ContentPtrIsNull,
    StringPtrIsNull,
    GitOidPtrIsNull,
    NotValidUtf8,
    NotValidUrl,
    NotGitOidUrl,
    CouldNotConstructUrl,
    StringHadInteriorNul,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Error::ContentPtrIsNull => write!(f, "data pointer is null"),
            Error::StringPtrIsNull => write!(f, "string pointer is null"),
            Error::GitOidPtrIsNull => write!(f, "GitOID pointer is null"),
            Error::NotValidUtf8 => write!(f, "string is not valid UTF-8"),
            Error::NotValidUrl => write!(f, "string is not a valid URL"),
            Error::NotGitOidUrl => write!(f, "string is not a valid GitOID URL"),
            Error::CouldNotConstructUrl => write!(f, "could not construct URL"),
            Error::StringHadInteriorNul => write!(f, "string had interior NUL byte"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

/// An error message extracted from an `AnyError`.
///
/// This is part of the implement of the LAST_ERROR mechanism, which takes any `AnyError`,
/// attempts to extract an `ErrorMsg` out of it, and then stores the resulting string
/// (from the `ToString` impl implies by `Display`) as the LAST_ERROR message.
#[derive(Debug)]
enum ErrorMsg {
    /// A successfully-extracted message.
    Known(String),

    /// Could not extract a message, so the error is unknown.
    Unknown,
}

impl Display for ErrorMsg {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            ErrorMsg::Known(s) => write!(f, "{}", s),
            ErrorMsg::Unknown => write!(f, "an unknown error occured"),
        }
    }
}

impl From<Box<dyn Any + Send + 'static>> for ErrorMsg {
    fn from(other: Box<dyn Any + Send + 'static>) -> ErrorMsg {
        if let Some(s) = other.downcast_ref::<String>() {
            ErrorMsg::Known(s.clone())
        } else if let Some(s) = other.downcast_ref::<&str>() {
            ErrorMsg::Known(s.to_string())
        } else {
            ErrorMsg::Unknown
        }
    }
}
