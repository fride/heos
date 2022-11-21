use axum::extract::{Path, Query};
use axum::Extension;

use serde::Deserialize;
use tracing::info;

use heos_api::types::Range;
use heos_api::HeosDriver;

use crate::error::AppError;

use crate::views::pages::music_containers::BrowseMusicContainerPage;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Params {
    #[serde(default)]
    start: Option<u16>,
    #[serde(default)]
    end: Option<u16>,
}

pub async fn browse_music_container(
    Query(params): Query<Params>,
    Path((source_id, container_id)): Path<(i64, String)>,
    Extension(driver): Extension<HeosDriver>,
) -> Result<BrowseMusicContainerPage, AppError> {
    info!("Enter browse_container");
    let range = match (params.start, params.end) {
        (Some(start), Some(end)) => Range { start, end },
        (Some(start), None) => Range { start, end: start + 10 },
        (None, Some(end)) => Range { start: 0, end },
        _ => Range::default(),
    };
    let items = driver
        .browse_music_containers(&source_id, &container_id, &range.clone())
        .await?;
    Ok(BrowseMusicContainerPage {
        items: items.items,
        source_id,
        count: items.count,
        returned: items.returned,
        container_id,
        range,
    })
}
