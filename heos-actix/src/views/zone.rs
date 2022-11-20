use crate::domain::zone::Zone;
use crate::views::page;
use heos_api::types::browse::MusicSource;
use heos_api::types::player::{NowPlayingMedia, QueueEntry};
use maud::{html, Markup};

pub fn zone_page(zones: Vec<Zone>, _music_sources: Vec<MusicSource>) -> Markup {
    page(
        "H E O S - Zones",
        "Zones".to_string(),
        html! {
            div class="zones" id="zones" {
                @for zone in &zones {
                    (zone_list_item(&zone))
                }
            }
        },
    )
}

pub fn zone_now_playing(now_playing: &NowPlayingMedia) -> Markup {
    html! {
        div class="zone-list-item__now-playing__icon" {
            img src=(now_playing.image_url) height="64px" width="64px";
        }
        p class="zone-list-item__now-playing__song" {(now_playing.song) }
        p class="zone-list-item__now-playing__album" { (now_playing.album) }
        p class="zone-list-item__now-playing__artist" { (now_playing.artist) }
    }
}

pub fn zone_item_actions(zone: &Zone) -> Markup {
    let edit_link = format!("/zones/{}/edit_members", zone.id());
    let target = format!("#{}", zone.zone_id());
    html! {
        div class="zone-list-item__actions" {
            a  href="{{edit_link}}" hx-swap="outerHTML"
               hx-select=(target)
               hx-target=(target)
               hx-get=(edit_link) {
                i class="fa fa-edit" aria-hidden="true" {}
            }
        }
    }
}

pub fn zone_item_media_controls(_zone: &Zone) -> Markup {
    html! {
        div class="zone-list-item__media_control" {
            button value="backward" {
                i class="fa fa-fast-backward" aria-hidden="true" {}
            }
            button value="play_pause" {
                i class="fa play" aria-hidden="true" {}
            }
            button value="forward" {
                i class="fa fa-fast-forward" aria-hidden="true" {}
            }

        }
    }
}

pub fn zone_list_item(zone: &Zone) -> Markup {
    html! {
        div class="zone-list-item" id=(format!("zone{}", zone.id())) {
            h3 class="zone-list-item__heading" {
                a href=(format!("/zones/{}", zone.id())) { (zone.name()) }
            }
            div class="zone-list-item__now-playing" {
                @if let Some(now_playing) = zone.now_playing() {
                    (zone_now_playing(now_playing))
                } @else {
                    div { ("Nothing here to hear")}
                }
            }
            (zone_item_actions(&zone))
            (zone_item_media_controls(&zone))
        }
    }
}

pub fn zone_detail_page(zone: Zone, _sources: Vec<MusicSource>, queue: Vec<QueueEntry>) -> Markup {
    page(
        "H E O S",
        "Zone".to_string(),
        html! {
            div.zone id="zone" {
                div.zone_heading {
                        h3 { (zone.name()) }
                     }
                div.zone__now-playing {
                @if let Some(now_playing) = zone.now_playing() {
                        (zone_now_playing(now_playing))
                    } @else {
                        div { ("Nothing here to hear")}
                    }
                }
                div.queue {
                    @for entry in queue {
                        div {
                           (entry.song)
                        }
                    }
                }
            }
        },
    )
}
