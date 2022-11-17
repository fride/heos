use crate::config::Config;
use anyhow::Context;
use axum::handler::Handler;
use axum::routing::{get_service, MethodRouter};
use axum::Router;
use heos_api::HeosDriver;
use tower_http::services::ServeDir;

mod browse;
mod error;
mod login;
mod players;
mod zones;

pub async fn serve(config: Config, driver: HeosDriver) -> anyhow::Result<()> {
    let app = router(config, driver).fallback(error::code_404.into_service());

    println!("Got up and running!");
    axum::Server::bind(&"0.0.0.0:8080".parse()?)
        .serve(app.into_make_service())
        .await
        .context("error running HTTP server")
}

fn router(config: Config, driver: HeosDriver) -> Router {
    let serve_dir = get_service(ServeDir::new("./statics")).handle_error(error::handle_error);
    // This is the order that the modules were authored in.
    browse::router(driver.clone())
        .merge(login::router(driver))
        .nest("/assets", serve_dir)
}
