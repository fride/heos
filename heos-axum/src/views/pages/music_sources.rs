use crate::views::browse::render_media_list_item;
use crate::views::pages::page;
use axum::response::{IntoResponse, Response};
use heos_api::types::browse::{BroseSourceItem, MusicSource};
use heos_api::types::SourceId;
use maud::{html, Markup};

pub struct BrowseMusicSourcePage {
    pub contents: Vec<BroseSourceItem>,
    pub source_id: SourceId,
    pub base_uri: String,
}

impl IntoResponse for BrowseMusicSourcePage {
    fn into_response(self) -> Response {
        self.render_html().into_response()
    }
}

impl BrowseMusicSourcePage {
    fn render(&self, item: &BroseSourceItem) -> Markup {
        match item {
            BroseSourceItem::HeosService(service) => {
                html!({
                    li {
                        a href=(format!("/sources/{}/browse", service.sid)) {
                            ( service.name )
                        }
                    }
                })
            }
            BroseSourceItem::BrowsableMedia(media) => {
                render_media_list_item(media, &self.source_id)
            }
        }
    }

    pub fn render_html(&self) -> Markup {
        let mut items: Vec<Markup> = vec![];

        let html = html!({
            ul .media-list {
                @for item in &self.contents {
                    ( self.render(item) )
                }
            }
        });
        super::page(html)
    }
}

pub struct SourceDetailsPage {
    pub source: MusicSource,
}
impl IntoResponse for SourceDetailsPage {
    fn into_response(self) -> Response {
        html!({
            div {
                h3 { (self.source.name) }
                img src=(self.source.image_url) {}
                div {
                    p {
                        a href="/sources" {
                        ( "back" )
                    }
                    } p {
                        a href =(format!("/sources/{}/browse", self.source.sid)) {
                            ( "browse" )
                        }
                    }
                }
            }
        })
        .into_response()
    }
}

pub struct MusicSourcesPages {
    pub base_uri: String,
    pub music_sources: Vec<MusicSource>,
}

impl MusicSourcesPages {
    pub fn render_html(&self) -> Markup {
        page(html!({
            ul .music-sources {
                @for source in &self.music_sources {
                    li {
                        img src=(source.image_url) height="32px" {}
                        a href=( format!("{}/sources/{}/browse", self.base_uri, source.sid)) {
                            ( source.name )
                        }
                    }
                }
            }
        }))
    }
}

impl IntoResponse for MusicSourcesPages {
    fn into_response(self) -> Response {
        self.render_html().into_response()
    }
}
