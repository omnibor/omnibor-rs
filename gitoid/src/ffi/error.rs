//! Errors arising from FFI code.
//!
//! This module contains four related parts:
//!
//! - A thread-local-storage-allocated value containing any error messages
//!   set by errors in FFI code, along with a getter and setter function.
//! - A mechanism for catching panics and recording error messages into that
//!   thread-local storage.
//! - An error type (plus trait impls) specific to FFI code.
//! - An "error message" type to assist capturing messages from panics.
//!
//! Together, these provide a consistent mechanism for collecting and reporting
//! errors to users of the `gitoid` FFI.

use crate::error::Error as GitOidError;
use core::any::Any;
use core::cell::RefCell;
use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result as FmtResult;
use core::panic::UnwindSafe;
use core::str::Utf8Error;
use std::error::Error as StdError;
use std::ffi::NulError;
use std::panic::catch_unwind;
use url::ParseError as UrlError;

thread_local! {
    // The last error to have been reported by the FFI code.
    /// cbindgen:ignore
    #[doc(hidden)]
    static LAST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
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
            set_error_msg(match err.source() {
                None => err.to_string(),
                Some(source_err) => {
                    format!("{}: {}", err, source_err)
                }
            });
            None
        }
        Err(err) => {
            // We have a `Box<dyn Any + Send + 'static>`
            set_error_msg(ErrorMsg::from(err).to_string());
            None
        }
    }
}

/// An Error arising from FFI code.
#[derive(Debug)]
pub(crate) enum Error {
    ContentPtrIsNull,
    StringPtrIsNull,
    GitOidPtrIsNull,
    Utf8UnexpectedEnd,
    Utf8InvalidByte(usize, usize),
    NotValidUrl(UrlError),
    NotGitOidUrl(GitOidError),
    StringHadInteriorNul(usize),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Error::ContentPtrIsNull => write!(f, "data pointer is null"),
            Error::StringPtrIsNull => write!(f, "string pointer is null"),
            Error::GitOidPtrIsNull => write!(f, "GitOID pointer is null"),
            Error::Utf8UnexpectedEnd => write!(f, "UTF-8 byte sequence ended unexpectedly"),
            Error::Utf8InvalidByte(start, len) => write!(
                f,
                "invalid {}-byte UTF-8 sequence, starting at byte {}",
                len, start
            ),
            Error::NotValidUrl(_) => write!(f, "string is not a valid URL"),
            Error::NotGitOidUrl(_) => write!(f, "string is not a valid GitOID URL"),
            Error::StringHadInteriorNul(loc) => {
                write!(f, "string had interior NUL at byte {}", loc)
            }
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::NotValidUrl(e) => Some(e),
            Error::NotGitOidUrl(e) => Some(e),
            _ => None,
        }
    }
}

impl From<Utf8Error> for Error {
    fn from(utf8_error: Utf8Error) -> Error {
        match utf8_error.error_len() {
            None => Error::Utf8UnexpectedEnd,
            Some(len) => Error::Utf8InvalidByte(utf8_error.valid_up_to(), len),
        }
    }
}

impl From<UrlError> for Error {
    fn from(url_error: UrlError) -> Error {
        Error::NotValidUrl(url_error)
    }
}

impl From<GitOidError> for Error {
    fn from(gitoid_error: GitOidError) -> Error {
        Error::NotGitOidUrl(gitoid_error)
    }
}

impl From<NulError> for Error {
    fn from(nul_error: NulError) -> Error {
        Error::StringHadInteriorNul(nul_error.nul_position())
    }
}

/// An error message arising from a panic.
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
