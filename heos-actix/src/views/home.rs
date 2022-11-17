use crate::domain::zone::Zone;
use crate::views::page;
use crate::views::zone::zone_list_item;
use heos_api::types::browse::MusicSource;
use maud::{html, Markup};

pub fn home(zones: Vec<Zone>, music_sources: Vec<MusicSource>) -> Markup {
    page(
        "H E O S - Player",
        "Music Sources".to_string(),
        html! {
            div class="zones" id="zones" {
                @for zone in &zones {
                    (zone_list_item(&zone))
                }
            }
            div class="music_sources" {
                @for source in music_sources {
                    div {
                        (source.name)
                        img src=(source.image_url) width="128px" {}
                    }
                }
            }
        },
    )
}
