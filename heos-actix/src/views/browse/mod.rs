use crate::views::ToHttpResponse;
use actix_web::{HttpRequest, HttpResponse};
use heos_api::types::browse::{BroseSourceItem, BrowsableMedia, HeosService, MusicSource};
use heos_api::types::{ContainerId, MediaId, SourceId};
use maud::{html, Markup};
use rust_hall::HalResource;

pub fn media_url(media: &BrowsableMedia, parent_id: &SourceId) -> String {
    match (&media.container_id, &media.mid) {
        (Some(cid), Some(mid)) => format!("/api/browse/{}/{}/{}", parent_id, &cid, &mid),
        (Some(cid), None) => format!("/api/browse/{}/{}", parent_id, &cid),
        (None, Some(mid)) => format!("/api/browse/{}/{}", parent_id, &mid),
        _ => "./".to_string(),
    }
}

fn get_source_link(parent_id: &SourceId, source: &BroseSourceItem) -> String {
    match source {
        BroseSourceItem::BrowsableMedia(ms) => media_url(&ms, &parent_id),
        BroseSourceItem::HeosService(hs) => {
            format!("/api/browse/{}/", &hs.sid)
        }
    }
}

pub fn render_browsable_media(media: &BrowsableMedia, parent_id: SourceId) -> Markup {
    let url = media_url(&media, &parent_id);
    html!({
        li {
            img src=( media.image_url) height="64px" {}
            a href=(url) {
                ( media.name )
            }
        }
    })
}

pub struct BrowseMusicSourcesResource(Vec<MusicSource>);

impl BrowseMusicSourcesResource {
    pub fn new(sources: Vec<MusicSource>) -> Self {
        Self(sources)
    }
}

fn source_li(source: &MusicSource, req: &HttpRequest) -> Markup {
    let url = req.url_for("browse", &[source.sid.to_string()]).unwrap();
    html!({
        li {
            img src=(source.image_url) height="64px"  {}
            a href = (url.to_string()) { ( source.name ) }
        }
    })
}

impl ToHttpResponse for BrowseMusicSourcesResource {
    fn to_html(&self, req: &HttpRequest) -> HttpResponse {
        let body = html!({
            h2 { ( "Aktive Musik Quellen" ) }
            ul .music-sources {
                @for source in self.0.iter().filter(|p| p.available) {
                    ( source_li(source, req) )
                }
            }
            h2 { ("Nicht aktive Quellen")}
            ul .music-sources {
                @for source in self.0.iter().filter(|p| !p.available) {
                    ( source_li(source, req) )
                }
            }
        });
        HttpResponse::Ok()
            .content_type(mime::TEXT_HTML_UTF_8)
            .body(body.into_string())
    }

    fn to_json(&self, req: &HttpRequest) -> HttpResponse {
        let mut resource = HalResource::with_self(req.url_for_static("music_sources").unwrap());
        let response = self.0.iter().cloned().fold(resource, |hal, music_source| {
            let embedded = HalResource::with_self(
                req.url_for("music_source", &[music_source.sid.to_string()])
                    .unwrap(),
            )
            .add_object(music_source);
            hal.with_embedded("music_sources", embedded)
        });
        HttpResponse::Ok().json(response)
    }
}

pub struct MusicSourceContentsResource(SourceId, Vec<BroseSourceItem>);

impl MusicSourceContentsResource {
    pub fn new(source_id: SourceId, things: Vec<BroseSourceItem>) -> Self {
        Self(source_id, things)
    }

    fn render_media(&self, media: &BrowsableMedia) -> Markup {
        html!({
            li {
                a href = ( media_url(&media, &self.0) ) { ( media.name )  }
            }
        })
    }
    fn render_source(&self, source: &HeosService, request: &HttpRequest) -> Markup {
        html!({
            li {
                a href = ( format!("/api/browse/{}", &source.sid) ) { ( source.name )  }
            }
        })
    }
}

impl ToHttpResponse for MusicSourceContentsResource {
    fn to_html(&self, req: &HttpRequest) -> HttpResponse {
        let lis = self.1.iter().map(|source| match source {
            BroseSourceItem::BrowsableMedia(media) => self.render_media(media),
            BroseSourceItem::HeosService(source) => self.render_source(source, &req),
            _ => html!({ p { ( " M A D N E S S" )}}),
        });
        let body = html!({
            ul {
                @for li in lis {
                    ( li )
                }
            }
        });
        HttpResponse::Ok()
            .content_type(mime::TEXT_HTML_UTF_8)
            .body(body.into_string())
    }

    fn to_json(&self, req: &HttpRequest) -> HttpResponse {
        let mut resource =
            HalResource::with_self(req.url_for("music_sources", [self.0.to_string()]).unwrap());
        let parent_id = self.0.clone();
        let response = self.1.iter().cloned().fold(resource, |hal, music_source| {
            let embedded = HalResource::with_self(get_source_link(&parent_id, &music_source))
                .add_object(music_source);
            hal.with_embedded("music_sources", embedded)
        });
        HttpResponse::Ok().json(response)
    }
}

pub struct BrowseContainerResource {
    pub source_id: SourceId,
    pub container_id: ContainerId,
    pub media: Vec<BrowsableMedia>,
}

impl BrowseContainerResource {
    pub fn new(
        source_id: SourceId,
        container_id: ContainerId,
        media: Vec<BrowsableMedia>,
    ) -> BrowseContainerResource {
        Self {
            source_id,
            container_id,
            media,
        }
    }
}

impl ToHttpResponse for BrowseContainerResource {
    fn to_html(&self, req: &HttpRequest) -> HttpResponse {
        let body = html!({
            ul {
                @for media in &self.media {
                    li {
                        ( media.name )
                    }
                }
            }
        });
        HttpResponse::Ok()
            .content_type(mime::TEXT_HTML_UTF_8)
            .body(body.into_string())
    }

    fn to_json(&self, req: &HttpRequest) -> HttpResponse {
        let mut resource = HalResource::with_self(
            req.url_for("music_sources", [self.source_id.to_string()])
                .unwrap(),
        );
        let response = self
            .media
            .iter()
            .cloned()
            .fold(resource, |hal, music_source| {
                let embedded = HalResource::with_self("").add_object(music_source);
                hal.with_embedded("service", embedded)
            });
        HttpResponse::Ok().json(response)
    }
}
