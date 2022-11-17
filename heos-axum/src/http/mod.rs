use std::time::{Duration, SystemTime};
use heos_api::HeosDriver;
use crate::config::Config;
use anyhow::Context;
use axum::{Router, TypedHeader};
use axum::extract::Path;
use axum::handler::Handler;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use headers::{ContentType, Expires};
use heos_api::error::HeosError;

mod browse;
mod login;

pub enum AppError {
    HeosError(HeosError),
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
        };
        (status, error_message).into_response()
    }
}


#[derive(Clone)]
pub struct ApiContext {
    pub driver: HeosDriver,
}

pub async fn serve(config: Config, driver: HeosDriver) -> anyhow::Result<()> {
    let app = api_router(config, driver.clone())
        .merge(login::router(driver))
        .fallback(code_404.into_service());
    println!("Got up and running!");
    axum::Server::bind(&"0.0.0.0:8080".parse()?)
        .serve(app.into_make_service())
        .await
        .context("error running HTTP server")
}

fn api_router(config: Config, driver: HeosDriver) -> Router {
    // This is the order that the modules were authored in.
    browse::router(driver)
    // .merge(profiles::router())
    // .merge(articles::router())
}

pub async fn code_404() -> impl IntoResponse {
    error_response(
        StatusCode::NOT_FOUND,
        "The resource you requested can't be found.",
    )
}

fn error_response(
    status_code: StatusCode,
    message: &str,
) -> impl IntoResponse + '_ {
    (status_code, format!("{}", message))
}
