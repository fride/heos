use anyhow::Error;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tracing::error;

use heos_api::error::HeosError;

pub enum AppError {
    HeosError(HeosError),
    InternalError(anyhow::Error),
    NotFound,
}

impl From<anyhow::Error> for AppError {
    fn from(err: Error) -> Self {
        AppError::InternalError(err)
    }
}
/// This makes it possible to use `?` to automatically convert a `UserRepoError`
/// into an `AppError`.
impl From<HeosError> for AppError {
    fn from(inner: HeosError) -> Self {
        AppError::HeosError(inner)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::HeosError(HeosError::InternalError(text)) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("You found a bug.\n {:#?}", text),
            ),
            AppError::HeosError(HeosError::InvalidCommand { command, eid: _, text }) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Heos is nasty! It failed to execute {}\n {} ", command, text),
            ),
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                "The thing that should not be can not be found!".to_string(),
            ),
            AppError::InternalError(err) => {
                error!("Well this sucks! {:#?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong".to_string())
            },
            AppError::HeosError(HeosError::NoDeviceFound) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "No HEOS devices found".to_string())
            },
        };
        (status, error_message).into_response()
    }
}
