use std::sync::{MutexGuard, PoisonError};

/// Unified error type for resource handler operations
#[derive(thiserror::Error, Debug)]
pub enum ResourceHandlerError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid URL: {0}")]
    UrlParseError(String),

    #[error("Failed to acquire state lock: {0}")]
    LockError(String),

    #[error("Resource state is not initialized")]
    StateNotInitializedError,

    #[error("Error opening file '{0}': {1}")]
    OpenFileError(std::path::PathBuf, String),

    #[error("Failed to process post data for URL '{0}': {1}")]
    PostDataError(String, String),

    #[error("Failed to register scheme {0}")]
    RegisterSchemeError(String),

    #[error("An error has occurred: {0}")]
    InternalError(String),
}

impl<T> From<PoisonError<MutexGuard<'_, T>>> for ResourceHandlerError {
    fn from(error: PoisonError<MutexGuard<'_, T>>) -> Self {
        Self::LockError(error.to_string())
    }
}
