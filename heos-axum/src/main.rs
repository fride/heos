use std::time::{Duration, SystemTime};

use axum::{
    extract::Path,
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
    Router,
    routing::get, Server, TypedHeader,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use heos_axum::config;
use heos_axum::controllers;

use crate::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "heos_axum=debug,heos_api=info,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    tracing::debug!("listening ");
    let diver = heos_api::HeosDriver::new("192.168.178.34:1255").await?;
    println!("Got Driver");
    controllers::serve(Config, diver).await?;
    Ok(())
}
