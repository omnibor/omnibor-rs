use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::io::Error as IoError;
use url::ParseError as UrlError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    BadLength { expected: usize, actual: usize },
    Url(UrlError),
    Io(IoError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::BadLength { expected, actual } => {
                write!(f, "expected length {}, actual length {}", expected, actual)
            }
            Error::Url(e) => write!(f, "{}", e),
            Error::Io(e) => write!(f, "{}", e),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::BadLength { .. } => None,
            Error::Url(e) => Some(e),
            Error::Io(e) => Some(e),
        }
    }
}

impl From<UrlError> for Error {
    fn from(e: UrlError) -> Error {
        Error::Url(e)
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Error {
        Error::Io(e)
    }
}
