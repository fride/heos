use std::sync::Arc;
use axum::{async_trait, http::StatusCode, response::{IntoResponse, Response}, routing::{get, post}, Json, Router, Extension};
use heos_api::HeosDriver;
use crate::config::Config;

pub mod listing;

pub fn router(driver: HeosDriver) -> Router {
    Router::new()
        .route("/sources/:source_id/containers/:container_id"
               , get(listing::browse_music_container))
        .route("/sources/:source_id/browse"
               , get(listing::browse_music_source))
        .route("/sources"
               , get(listing::list_music_sources))
        .layer(Extension(driver))
}
