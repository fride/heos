use std::collections::BTreeMap;
use axum::{Extension, Router};
use axum::response::IntoResponse;
use axum::routing::get;
use maud::html;
use heos_api::HeosDriver;
use heos_api::types::player::{HeosPlayer, PlayerInfo};
use heos_api::types::{GroupId, Level, PlayerId};
use heos_api::types::group::Group;
use crate::models::zones::Zones;
use crate::views::pages::page;
use crate::views::zones::render_zone;

pub struct ZonesPage{
    pub zones: Zones
}
impl ZonesPage {
    pub fn new(mut players: Vec<HeosPlayer>, groups: Vec<Group>) -> Self {
        let zones : Zones = (players, groups).into();
        Self{
            zones
        }
    }
}
pub async fn show_zones(
    Extension(driver): Extension<HeosDriver>) -> impl IntoResponse {
    let groups = driver.groups();
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
