use crate::error::AppError;

use crate::views::pages::page;
use crate::views::zones::edit::EditZoneMembers;
use crate::views::zones::listing::ZonesPage;
use anyhow::anyhow;
use anyhow::Context;
use axum::extract::Path;
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::{Extension, Form, Router};

use heos_api::types::player::HeosPlayer;
use heos_api::types::PlayerId;
use heos_api::HeosDriver;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tracing::info;

pub async fn show_zones(Extension(driver): Extension<HeosDriver>) -> impl IntoResponse {
    let _groups = driver.groups();
    let pages = ZonesPage::new(driver.players(), driver.groups());
    page(pages.render_html())
}

pub async fn show_edit_zone_members(
    Path(zone_id): Path<i64>,
    Extension(driver): Extension<HeosDriver>,
) -> Result<EditZoneMembers, AppError> {
    info!("Start show_edit_zone_members");
    info!("Found group to edit");
    let mut players: BTreeMap<i64, HeosPlayer> = driver
        .players()
        .into_iter()
        .map(|p| (p.player_id, p))
        .collect();
    let player_to_edit = players.remove(&zone_id).ok_or(AppError::NotFound)?;
    let page = EditZoneMembers::new(player_to_edit, players.into_values());
    info!("Page: {:?}", &page.members);
    Ok(page)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChangeZoneMemberForm {
    #[serde(flatten)]
    pub members: BTreeMap<String, String>, // this is a bit shitty!
}

impl ChangeZoneMemberForm {
    pub fn get_selected_player_ids(&self) -> Result<Vec<PlayerId>, AppError> {
        let mut pids = vec![];
        for (key, _) in &self.members {
            let pid = key.parse().map_err(|e| {
                AppError::InternalError(anyhow!("Failed to parse {} as number", &e))
            })?;
            pids.push(pid);
        }
        Ok(pids)
    }
}
pub async fn change_zone_members(
    Path(zone_id): Path<i64>,
    Form(form): Form<ChangeZoneMemberForm>,
    Extension(driver): Extension<HeosDriver>,
) -> Result<Redirect, AppError> {
    driver
        .create_group(zone_id, form.get_selected_player_ids()?)
        .await
        .context("Failed to set groups")?;
    info!("Start change_zone_members: {:?}", &form);
    // this leads to a post to this location!?
    Ok(Redirect::to("/zones"))
}

pub fn router(driver: HeosDriver) -> Router {
    Router::new()
        .route("/zones", get(show_zones))
        .route("/zones/:zone_id/edit-members", get(show_edit_zone_members))
        .route("/zones/:zone_id/", post(change_zone_members))
        .layer(Extension(driver))
}
