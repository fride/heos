use std::time::{Duration, SystemTime};
use anyhow::Context;
use axum::extract::Path;
use axum::handler::Handler;
use axum::response::IntoResponse;
use axum::{Router, TypedHeader};
use axum::routing::{get};
use headers::{ContentType, Expires};
use tower_http::trace::TraceLayer;
use tracing::info;

use heos_api::HeosDriver;

use crate::config::Config;

// this is generated before build
use crate::templates::statics::StaticFile;

mod browse;
mod error;
mod login;
mod players;
mod zones;

pub async fn serve(config: Config, driver: HeosDriver) -> anyhow::Result<()> {
    let app = router(config, driver).fallback(error::code_404.into_service())
        // See https://docs.rs/tower-http/0.1.1/tower_http/trace/index.html for more details.
        .layer(TraceLayer::new_for_http());
    info!("Got up and running!");
    axum::Server::bind(&"0.0.0.0:8080".parse()?)
        .serve(app.into_make_service())
        .await
        .context("error running HTTP server")
}

fn router(_config: Config, driver: HeosDriver) -> Router {
    // This is the order that the modules were authored in.
    browse::router(driver.clone())
        .route("/assets/:filename", get(static_files))
        .merge(login::router(driver.clone()))
        .merge(players::router(driver.clone()))
        .merge(zones::router(driver))
}

/// Handler for static files.
/// Create a response from the file data with a correct content type
/// and a far expires header (or a 404 if the file does not exist).
/// from https://github.com/kaj/ructe/blob/master/examples/axum/src/main.rs
async fn static_files(Path(filename): Path<String>) -> impl IntoResponse {
    /// A duration to add to current time for a far expires header.
    static FAR: Duration = Duration::from_secs(180 * 24 * 60 * 60);
    match StaticFile::get(&filename) {
        Some(data) => {
            let far_expires = SystemTime::now() + FAR;
            (
                TypedHeader(ContentType::from(data.mime.clone())),
                TypedHeader(Expires::from(far_expires)),
                data.content,
            ).into_response()
        }
        None => error::code_404().await.into_response(),
    }
}
