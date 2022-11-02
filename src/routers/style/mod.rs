use actix_web::http::header::ContentType;
use actix_web::HttpResponse;

pub async fn main_css() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/css")
        .body(include_str!("style.css"))
}
