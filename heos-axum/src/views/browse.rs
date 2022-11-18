use axum::response::{IntoResponse, Response};
use maud::{html, Markup};

use heos_api::types::{ContainerId, Range, SourceId};
use heos_api::types::browse::{BroseSourceItem, BrowsableMedia, MusicSource};
use crate::templates::statics::*;

use crate::views::pages::page;

pub fn render_media_list_item(item: &BrowsableMedia, source_id: &SourceId) -> Markup {
    let description: Markup = match (&item.artist, &item.album) {
        (Some(artist), Some(album)) => html!({
            p .name { (item.name) }
            p .artist { (artist) }
            p .album { (album) }
        }),
        (Some(artist), None) => html!({
            p .name { (item.name) }
            p .artist { (artist) }
        }),
        (_, _) => html!({
            p .name  { (item.name ) }
        }),
    };
    let image_url  = if item.image_url.is_empty() {
        format!("/assets/{}", &folder_svg.name)
    } else {
        item.image_url.clone()
    };
    html!({
        li {
            .media-list__media-item{
                @if let Some(cid) = &item.container_id {
                img src=(image_url) height="32px" {}
                a href=( format!("/sources/{}/containers/{}/", source_id, cid) ) {
                    ( description )
                }
                } @else {
                    img src=(image_url) height="32px" {}
                    ( description )
                }
            }
        }
    })
}
