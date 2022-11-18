use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Router};
use heos_api::types::player::HeosPlayer;
use heos_api::HeosDriver;
use maud::html;

use crate::models::zones::Zones;
use crate::views::pages::page;
use crate::views::zones::render_zone;
use heos_api::types::group::Group;

pub struct ZonesPage {
    pub zones: Zones,
}
impl ZonesPage {
    pub fn new(players: Vec<HeosPlayer>, groups: Vec<Group>) -> Self {
        let zones: Zones = (players, groups).into();
        Self { zones }
    }
}
pub async fn show_zones(Extension(driver): Extension<HeosDriver>) -> impl IntoResponse {
    let _groups = driver.groups();
    let pages = ZonesPage::new(driver.players(), driver.groups());
    page(html!({
        div .zones{
            ol {
                @for zone in pages.zones.iter() {
                    li {
                        (render_zone(&zone))
                    }
                }
            }
        }
    }))
}

pub fn router(driver: HeosDriver) -> Router {
    Router::new()
        .route("/zones", get(show_zones))
        .layer(Extension(driver))
}
