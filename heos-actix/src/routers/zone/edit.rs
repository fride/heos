use crate::domain::zone::Zone;
use actix_web::http::header::ContentType;
use actix_web::web::Path;
use actix_web::{web, HttpResponse};
use heos_api::types::PlayerId;
use heos_api::HeosDriver;
use maud::html;

pub struct ZoneMemberModel {
    pub id: PlayerId,
    pub name: String,
    pub selected: bool,
}
pub async fn edit_zone_members_form(path: Path<i64>, driver: web::Data<HeosDriver>) -> HttpResponse {
    let zone_id = path.into_inner();
    let players = driver.players();
    let zones = Zone::get_zones(&driver);
    let leader = zones.iter().find(|p| p.id() == zone_id).unwrap();
    let members = players.iter().filter_map(|player| {
        if player.player_id == leader.id() {
            None
        } else if leader.contains_member(player.player_id) {
            Some(ZoneMemberModel {
                id: player.player_id,
                name: player.name.clone(),
                selected: true,
            })
        } else {
            Some(ZoneMemberModel {
                id: player.player_id.clone(),
                name: player.name.clone(),
                selected: false,
            })
        }
    });

    let html = html! {
        form method="post" action=(format!("/zones/{}", zone_id)) id=(format!("zone{}", zone_id))
            hx-post=(format!("/zones/{}", zone_id))
            hx-target="#zones"
            hx-select="#zones"
            hx-swap="outerHTML" {
            div class="zone-list-item" {
                div class="zone-list-item__heading" {
                    h3 { (leader.name()) }
                }
                div class="zone-list-item__members" {
                    @for member in members {
                        div style="margin-left: 2em;" {
                            label for="{{member.id}}" { (member.name) }
                            input type="checkbox" id=(member.id) name=(member.id) checked?[member.selected] {}
                        }
                    }
                }
                div class="zone-list-item__actions" {
                    img height="32px" width="32px" class="htmx-indicator" src="https://raw.githubusercontent.com/SamHerbert/SVG-Loaders/master/svg-loaders/grid.svg" {}
                }
                div class="zone-list-item__media_control" {
                    span margin-left="0.5em"{
                        button name="change_groups" id="change_groups" {
                            i class="fa fa-check" {}
                        }
                    }
                    span {
                        a href="/" {
                            i class="fa fa-times" {}
                        }
                    }
                }
            }
        }
    };
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html.into_string())
}
