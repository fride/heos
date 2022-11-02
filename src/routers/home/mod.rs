use actix_web::{http::header::ContentType, web, HttpRequest, HttpResponse};
use heos_api::HeosDriver;

use crate::domain::zone::Zone;
use crate::views::home::home as home_html;

pub async fn home(_req: HttpRequest, driver: web::Data<HeosDriver>) -> HttpResponse {
    // let edit_link = req.url_for(
    //     "edit_members", [])// format!("/zones{}/edit", zone.id());
    let zones = Zone::get_zones(&driver);
    let sources = driver.music_sources();

    let html = home_html(zones, sources);
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html.into_string())
}
