use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use heos_api::error::HeosError;

pub enum AppError {
    HeosError(HeosError),
    NotFound
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
            AppError::HeosError(HeosError::InternalError(text)) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("You found a bug.\n {:#?}", text))
            }
            AppError::HeosError(HeosError::InvalidCommand {command, eid, text}) => {
                (StatusCode::UNPROCESSABLE_ENTITY, format!("Heos is nasty! It failed to execute {}\n {} ",command, text))
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "The thing that should not be can not be found!".to_string())
        };
        (status, error_message).into_response()
    }
}
