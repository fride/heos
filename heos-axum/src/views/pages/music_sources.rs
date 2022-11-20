use axum::response::{IntoResponse, Response};
use maud::{html, Markup};

use heos_api::types::browse::{BrowsableMedia, HeosService, MusicSource};
use heos_api::types::SourceId;

use crate::views::browse::render_media_list_item;
use crate::views::pages::page;

pub struct BrowseMusicSourcePage {
    pub media_items: Vec<BrowsableMedia>,
    pub services: Vec<HeosService>,
    pub source_id: SourceId,
    pub base_uri: String,
}

impl IntoResponse for BrowseMusicSourcePage {
    fn into_response(self) -> Response {
        self.render_html().into_response()
    }
}

impl BrowseMusicSourcePage {
    pub fn render_html(&self) -> Markup {
        let html = html!({
            div {
                ol .media-list {
                    @for item in &self.media_items {
                        ( render_media_list_item(item, &self.source_id) )
                    }
                }
            }
            div {
                ol. media-list {
                    @for service in &self.services {
                         li {
                            div .media-list__heos-service {
                                img src=(service.image_url) height="32px" {}
                                a href=(format!("/sources/{}/browse", service.sid)) {
                                    ( service.name )
                                }
                            }
                        }
                    }
                }
            }
        });
        page(html)
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
            div .music-sources {
                @for source in &self.music_sources {
                     div {
                        a href=(format!("{}/sources/{}/browse", self.base_uri, source.sid)) alt=( source.name ) {
                            img src=(source.image_url) {}
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
