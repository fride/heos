use anyhow::Context;
use axum::extract::Path;
use axum::handler::Handler;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Router, TypedHeader};
use clap::builder::Str;
use headers::{ContentType, Expires};
use std::net::{IpAddr, Ipv4Addr};
use std::time::{Duration, SystemTime};
use tokio::signal;
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

#[derive(Clone)]
pub struct BaseUrl(String);

impl BaseUrl {
    pub fn new(base_url: String) -> Self {
        BaseUrl(base_url)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub async fn serve(config: Config, driver: HeosDriver) -> anyhow::Result<()> {
    let app = router(&config, driver)
        .fallback(error::code_404.into_service())
        // See https://docs.rs/tower-http/0.1.1/tower_http/trace/index.html for more details.
        .layer(TraceLayer::new_for_http());

    let local_addr = &config.get_local_addr();
    info!("Listening on {}", &local_addr);
    axum::Server::bind(local_addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("error running HTTP server")
}

fn router(config: &Config, driver: HeosDriver) -> Router {
    // This is the order that the modules were authored in.
    browse::router(driver.clone(), &config)
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
            )
                .into_response()
        },
        None => error::code_404().await.into_response(),
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("signal received, starting graceful shutdown");
}
