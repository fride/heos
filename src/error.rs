use std::num::ParseIntError;

use qs;
use thiserror::Error;
use tokio::sync::oneshot::error::RecvError;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum HeosErrorCode {
    UnrecognizedCommand = 1,
    InvalidId = 2,
    WrongNumberOfArguments = 3,
    RequestedDataNotAvailable = 4,
    ResourceCurrentlyNotAvailable = 5,
    InvalidCredentials = 6,
    CommandCouldNitBeExecuted = 7,
    UserNotLoggedIn = 8,
    ParameterOutOfRange = 9,
    UserNotFound = 10,
    InternalError = 11,
    SystemError = 12,
    ProcessingPreviousCommand = 13,
    MediaCantBePlayed = 14,
    OptionNotSupported = 15,
    Unknown,
}

#[derive(Error, Debug)]
pub enum HeosError {
    /// Represents a failure to read from input.
    #[error("Read error")]
    ReadError { source: std::io::Error },

    #[error("Parse error")]
    ParseError(#[from] serde_json::Error),

    #[error("Parse int error")]
    ParseIntError(#[from] ParseIntError),

    #[error("Parse Error")]
    ParserError { message: String },

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error("Network error.")]
    NetworkError { message: String },

    /// Represents all other cases of `std::io::Error`.
    #[error("Invalid command ")]
    InvalidCommand {
        command: String,
        message: String,
        // eid: HeosErrorCode,
        // text: String,
    },

    // remove this!
    #[error("Some damn error: {message} ")]
    Error { message: String },

    // does not handle tokio::sync::oneshot::error::RecvError?
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<&str> for HeosError {
    fn from(error: &str) -> Self {
        HeosError::Error {
            message: error.to_owned(),
        }
    }
}

impl From<RecvError> for HeosError {
    fn from(_err: RecvError) -> Self {
        HeosError::Error {
            message: format!("{}", "failed to receive"),
        }
    }
}
impl From<String> for HeosError {
    fn from(error: String) -> Self {
        HeosError::Error { message: error }
    }
}

impl From<qs::Error> for HeosError {
    fn from(error: qs::Error) -> Self {
        HeosError::Error {
            message: format!("failed to parse message: {}", error),
        }
    }
}
