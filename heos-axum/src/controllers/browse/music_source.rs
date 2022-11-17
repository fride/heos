use axum::Extension;
use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use maud::html;
use heos_api::HeosDriver;
use heos_api::types::browse::MusicSource;
use crate::error::AppError;
use crate::views::pages::music_sources::{BrowseMusicSourcePage, MusicSourcesPages, SourceDetailsPage};

pub async fn source_details(
    Path(source_id): Path<i64>,
    Extension(driver): Extension<HeosDriver>) -> Result<SourceDetailsPage, AppError> {
    let source = driver.music_sources().into_iter().find(|s| s.sid == source_id)
        .ok_or(AppError::NotFound)?;
    Ok(SourceDetailsPage {source})
}

pub async fn list_music_sources(Extension(driver): Extension<HeosDriver>) -> Result<MusicSourcesPages, AppError> {
    let music_sources = driver.music_sources();
    Ok(MusicSourcesPages {
        base_uri: "http://localhost:8080".to_string(),
        music_sources,
    })
}

pub async fn browse_music_source(
    Path(source_id): Path<i64>,
    Extension(driver): Extension<HeosDriver>) -> Result<BrowseMusicSourcePage, AppError> {
    let contents = driver.browse(source_id).await?;
    Ok(BrowseMusicSourcePage { base_uri: "".to_string(), source_id, contents })
}
