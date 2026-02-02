use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
pub enum AppError {
    #[error("Operation cancelled by user")]
    Cancelled,

    #[error("Administrator privileges required")]
    AdminRequired,

    #[error("UAC prompt was cancelled")]
    UacCancelled,

    #[error("Script execution failed: {0}")]
    ScriptError(String),

    #[error("File operation failed: {0}")]
    FileError(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Timeout exceeded")]
    Timeout,

    #[error("IO error: {0}")]
    IoError(String),
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
