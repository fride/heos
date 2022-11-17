use axum::response::{IntoResponse, Response};
use heos_api::types::browse::{BroseSourceItem, BrowsableMedia, MusicSource};
use heos_api::types::{ContainerId, Range, SourceId};
use maud::{html, Markup};
use crate::views::pages::page;

pub fn render_media_list_item(item: &BrowsableMedia, source_id: &SourceId) -> Markup {
    let description : Markup = match (&item.artist, &item.album) {
        (Some(artist), Some(album)) => html!({
                p .name { (item.name) }
                p .artist { (artist) }
                p .album { (album) }
            }),
        (Some(artist), None) => html!({
                p .name { (item.name) }
                p .artist { (artist) }
            }),
        (_,_) => html!({
                p .name  { (item.name ) }
            })
    };

    html!({
            li {
                .media-list__item {
                    @if let Some(cid) = &item.container_id {
                    a href=( format!("/sources/{}/containers/{}/", source_id, cid) ) {
                        img src=(item.image_url) height="32px" {}
                        ( description )
                    }
                    } @else {
                        img src=(item.image_url) height="32px" {}
                        ( description )
                    }
                }
            }
        })
}
