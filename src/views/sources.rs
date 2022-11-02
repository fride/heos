use heos_api::types::browse::MusicSource;
use maud::{html, Markup};
use crate::domain::zone::Zone;
use crate::views::page;

pub fn sources_page(music_sources: Vec<MusicSource>) -> Markup {
    page("H E O S - Music Sources", "Music Sources".to_string(), html!{
        div class="music_sources" {
            @for source in music_sources {
                div {
                    (source.name)
                    img src=(source.image_url) width="128px" {}
                }
            }
        }
    })
}
