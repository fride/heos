use axum::Extension;
use axum::extract::Path;
use axum::response::{Response, IntoResponse, Html};
use heos_api::{HeosDriver, HeosResult};
use heos_api::types::browse::{BroseSourceItem, BrowsableMedia, HeosService, MediaType, MusicSource};
use heos_api::types::{ContainerId, MediaId, Range, SourceId};
use maud::{html, Markup};
use tracing::{info};
use crate::axum_ructe::render;
use crate::http::{ApiContext, AppError};

pub struct MusicSources {
    pub base_uri: String,
    pub music_sources: Vec<MusicSource>,
}

impl MusicSources {
    pub fn render_html(&self) -> Markup {
        html!({
            ul {
                @for source in &self.music_sources {
                    li {
                        img src=(source.image_url) height="32px" {}
                        a href=( format!("{}/sources/{}/browse", self.base_uri, source.sid)) {
                            ( source.name )
                        }
                    }
                }
            }
        })
    }
}

impl IntoResponse for MusicSources {
    fn into_response(self) -> Response {
        self.render_html().into_response()
    }
}

pub async fn list_music_sources(Extension(driver): Extension<HeosDriver>) -> Result<MusicSources, AppError> {
    let music_sources = driver.music_sources();
    Ok(MusicSources {
        base_uri: "http://localhost:8080".to_string(),
        music_sources,
    })
}


pub struct BrowseMusicSource {
    pub contents: Vec<BroseSourceItem>,
    pub source_id: SourceId,
    pub base_uri: String,
}

impl BrowseMusicSource {
    pub fn render_html(&self) -> Markup {
        let mut items: Vec<Markup> = vec![];
        let html = html!({
        ul {
            @for item in &self.contents {
                ( browse_container_list_item( self.source_id, item ) )
            }
        }
    });
        html
    }
}

pub async fn browse_music_source(
    Path(source_id): Path<i64>,
    Extension(driver): Extension<HeosDriver>) -> Result<Markup, AppError> {
    info!("Enter browse_container");
    let contents = driver.browse(source_id).await?;
    Ok(BrowseMusicSource { base_uri: "".to_string(), source_id, contents }.render_html())
}

fn browse_container_list_item(source_id: SourceId, item: &BroseSourceItem) -> Markup {
    match item {
        BroseSourceItem::HeosService(service) => heos_service_list_item(&source_id, &service),
        BroseSourceItem::BrowsableMedia(item) =>
            heos_media_list_item(&source_id, &item)
    }
}

fn heos_media_list_item(parent_id: &SourceId, media: &BrowsableMedia) -> Markup {
    let link = match (&media.container_id, &media.mid) {
        (Some(cid), Some(mid)) => "#".to_string(),
        (Some(cid), None) => format!("/sources/{}/containers/{}/", parent_id, cid),
        (None, None) => "#".to_string(),
        _ => "#".to_string()
    };
    html!({
        li {
            a href=(link) {
                (media.name)
            }
        }
    })
}

fn heos_service_list_item(parent_id: &SourceId, service: &HeosService) -> Markup {
    html!({
        li {
            a href=(format!("/sources/{}/browse", service.sid)) {
                ( service.name )
            }
        }
    })
}

pub struct BrowseMusicContainer {
    pub source_id: SourceId,
    pub parent_container_id: ContainerId,
    pub items: Vec<BrowsableMedia>,
    pub range: Range,
}

impl BrowseMusicContainer {
    pub fn render_html(&self) -> Markup {
        html!({
            a href="/sources/" { ( "Back to sources")}
            a href=( format!("/sources/{}/browse", self.source_id)) { ( "Back to source")}
            a href=( format!("/sources/{}/containers/{}", self.source_id, self.parent_container_id)) { ( "Back to parent container")}
            ul {
                @for item in &self.items {
                    ( heos_media_list_item( &self.source_id, item ) )
                }
            }
        })
    }
}

impl IntoResponse for BrowseMusicContainer {
    fn into_response(self) -> Response {
        self.render_html().into_response()
    }
}
pub async fn browse_music_container(
    Path((source_id, container_id)): Path<(i64, String)>,
    Extension(driver): Extension<HeosDriver>) -> Result<BrowseMusicContainer, AppError> {
    info!("Enter browse_container");
    let range = Range::default();
    let items = driver.browse_music_containers(&source_id, &container_id, &range.clone()).await?;
    Ok(BrowseMusicContainer {
        items,
        source_id,
        parent_container_id: container_id,
        range
    })
}


