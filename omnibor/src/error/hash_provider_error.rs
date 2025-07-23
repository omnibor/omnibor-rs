use std::{error::Error, fmt::Display};

/// Errors arising during hash provider initialization/
#[derive(Debug)]
pub enum HashProviderError {
    /// The chosen hash provider is not valid.
    InvalidHashProvider,
}

impl Display for HashProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashProviderError::InvalidHashProvider => {
                write!(f, "the chosen hash provider is not valid")
            }
        }
    }
}

impl Error for HashProviderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            HashProviderError::InvalidHashProvider => None,
        }
    }
}
