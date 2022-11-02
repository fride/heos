use crate::views;
use actix_web::http::header::ContentType;
use actix_web::{web, HttpRequest, HttpResponse};
use heos_api::HeosDriver;

pub async fn list(_req: HttpRequest, driver: web::Data<HeosDriver>) -> HttpResponse {
    let music_sources = driver.music_sources();
    let html = views::sources::sources_page(music_sources);
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html.into_string())
}
