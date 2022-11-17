use actix_web::{Either, HttpRequest, HttpResponse, web, Error, HttpMessage};
use actix_web::error::InternalError;
use actix_web::http::StatusCode;
use actix_web::web::Path;
use heos_api::error::HeosError;
use heos_api::HeosDriver;
use heos_api::types::browse::{BroseSourceItem, MusicSource};
use heos_api::types::{Range, SourceId};
use maud::{html, Markup};
use rust_hall::{HalResource, Link};
use crate::views::browse::{BrowseContainerResource, BrowseMusicSourcesResource, MusicSourceContentsResource};
use crate::views::ToHttpResponse;

type RegisterResult = Either<HttpResponse, Result<&'static str, Error>>;

pub async fn list(req: HttpRequest,
                  driver: web::Data<HeosDriver>) -> HttpResponse {
    let music_sources = BrowseMusicSourcesResource::new(driver.music_sources());
    music_sources.to_response(&req)
}


pub async fn details(
    req: HttpRequest,
    path: Path<i64>,
    driver: web::Data<HeosDriver>,
) -> Result<HttpResponse, InternalError<HeosError>> {
    let source_id: SourceId = path.into_inner();
    let browse_result = driver.browse(source_id)
        .await
        .map_err(|heos_err|
            InternalError::new(heos_err, StatusCode::INTERNAL_SERVER_ERROR))?;
    let music_sources = MusicSourceContentsResource::new(source_id, browse_result);
    Ok(music_sources.to_response(&req))
}

// here it gets ugly!
pub async fn container(
    req: HttpRequest,
    path: Path<(i64, String)>,
    driver: web::Data<HeosDriver>,
) -> HttpResponse {
    let (source_id, container_id) = path.into_inner();
    let music_sources = driver.browse_music_containers(&source_id, &container_id, &Range::default()).await.unwrap();
    BrowseContainerResource::new(source_id, container_id, music_sources).to_response(&req)
}
