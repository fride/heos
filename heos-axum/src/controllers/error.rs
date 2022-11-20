use axum::http::StatusCode;
use axum::response::IntoResponse;

pub async fn code_404() -> impl IntoResponse {
    error_response(
        StatusCode::NOT_FOUND,
        "The resource you requested can't be found.",
    )
}

fn error_response(status_code: StatusCode, message: &str) -> impl IntoResponse + '_ {
    (status_code, format!("{}", message))
}
