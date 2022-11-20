use std::sync::Arc;
use axum::extract::Path;
use axum::Extension;

use heos_api::types::browse::BroseSourceItem;
use heos_api::HeosDriver;
use crate::controllers::BaseUrl;

use crate::error::AppError;
use crate::views::pages::music_sources::{
    BrowseMusicSourcePage, MusicSourcesPages, SourceDetailsPage,
};

pub async fn source_details(
    Path(source_id): Path<i64>,
    Extension(driver): Extension<HeosDriver>,
) -> Result<SourceDetailsPage, AppError> {
    let source = driver
        .music_sources()
        .into_iter()
        .find(|s| s.sid == source_id)
        .ok_or(AppError::NotFound)?;
    Ok(SourceDetailsPage { source })
}

pub async fn list_music_sources(
    Extension(driver): Extension<HeosDriver>,
    Extension(baseUrl): Extension<Arc<BaseUrl>>,
) -> Result<MusicSourcesPages, AppError> {
    let music_sources = driver.music_sources();
    Ok(MusicSourcesPages {
        base_uri: baseUrl.as_str().to_string(),
        music_sources,
    })
}

pub async fn browse_music_source(
    Path(source_id): Path<i64>,
    Extension(driver): Extension<HeosDriver>,
) -> Result<BrowseMusicSourcePage, AppError> {
    let contents = driver.browse(source_id).await?;
    use itertools::{Either, Itertools};
    let (services, media_items) = contents.into_iter().partition_map(|item| match item {
        BroseSourceItem::HeosService(service) => Either::Left(service),
        BroseSourceItem::BrowsableMedia(media) => Either::Right(media),
    });
    Ok(BrowseMusicSourcePage {
        base_uri: "".to_string(),
        source_id,
        services,
        media_items,
    })
}
