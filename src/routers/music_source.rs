use actix_web::{HttpRequest, HttpResponse, web};
use actix_web::http::header::ContentType;
use heos_api::HeosDriver;
use crate::views;

pub async fn list(req: HttpRequest,
                  driver: web::Data<HeosDriver>) -> HttpResponse {
    let music_sources = driver.music_sources();
    let html = views::sources::sources_page(music_sources);
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html.into_string())
}
