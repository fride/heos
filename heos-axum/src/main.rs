use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use heos_axum::config;
use heos_axum::controllers;

use crate::config::Config;
use clap::Parser;
use heos_api::HeosDriver;

use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse();
    let rust_log = config
        .rust_log
        .clone()
        .unwrap_or_else(|| "heos_axum=debug,heos_api=info,tower_http=debug".into());
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(rust_log))
        .with(tracing_subscriber::fmt::layer())
        .init();
    info!("Starting ...");
    let diver = match config.heos_device_addr {
        Some(addr) => HeosDriver::new((addr, 1255)).await?,
        None => heos_api::find_driver().await?,
    };
    println!("Found driver, now starting http server");
    controllers::serve(config, diver).await?;
    Ok(())
}
