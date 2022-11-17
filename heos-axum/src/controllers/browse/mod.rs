use std::sync::Arc;
use axum::{async_trait, http::StatusCode, response::{IntoResponse, Response}, routing::{get, post}, Json, Router, Extension};
use heos_api::HeosDriver;
use crate::config::Config;

mod music_container;
mod music_source;

pub fn router(driver: HeosDriver) -> Router {
    Router::new()
        .route("/sources/:source_id/containers/:container_id"
               , get(music_container::browse_music_container))
        .route("/sources/:source_id/browse"
               , get(music_source::browse_music_source))
        .route("/sources/:source_id"
               , get(music_source::source_details))
        .route("/sources"
               , get(music_source::list_music_sources))
        .layer(Extension(driver))
}
