use std::sync::Arc;
use axum::{routing::get, Extension, Router};

use heos_api::HeosDriver;
use crate::config::Config;
use crate::controllers::BaseUrl;

mod music_container;
mod music_source;

pub fn router(driver: HeosDriver, config: &Config) -> Router {
    let base_url = Arc::new(BaseUrl::new(config.base_url.clone()));
    Router::new()
        .route(
            "/sources/:source_id/containers/:container_id",
            get(music_container::browse_music_container),
        )
        .route(
            "/sources/:source_id/browse",
            get(music_source::browse_music_source),
        )
        .route("/sources/:source_id", get(music_source::source_details))
        .route("/sources", get(music_source::list_music_sources))
        .layer(Extension(driver))
        .layer(Extension(base_url))
}
