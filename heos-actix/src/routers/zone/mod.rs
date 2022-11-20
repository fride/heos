mod create_group;
/// Routes for zones, lisy, edit details.
mod edit;

use crate::domain::zone::Zone;
use crate::views::zone::{zone_detail_page, zone_page};
use actix_web::http::header::ContentType;
use actix_web::web::Path;
use actix_web::{web, HttpRequest, HttpResponse};
pub use create_group::*;
pub use edit::*;
use heos_api::types::Range;
use heos_api::HeosDriver;

pub async fn list(_req: HttpRequest, driver: web::Data<HeosDriver>) -> HttpResponse {
    // let edit_link = req.url_for(
    //     "edit_members", [])// format!("/zones{}/edit", zone.id());
    let zones = Zone::get_zones(&driver);
    let sources = driver.music_sources();

    let html = zone_page(zones, sources);
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html.into_string())
}

pub async fn details(
    _req: HttpRequest,
    path: Path<i64>,
    driver: web::Data<HeosDriver>,
) -> HttpResponse {
    let player_id = path.into_inner();
    let zones = Zone::get_zones(&driver);
    if let Some(zone) = zones.into_iter().find(|p| p.id() == player_id) {
        let sources = driver.music_sources();
        let queue = driver
            .get_player_queue(player_id, Range::default())
            .await
            .unwrap();
        let html = zone_detail_page(zone, sources, queue);
        HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(html.into_string())
    } else {
        HttpResponse::NotFound().finish()
    }
}
