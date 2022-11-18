use axum::response::{IntoResponse, Response};
use maud::{html, Markup};
use tracing::info;

use heos_api::types::{ContainerId, Range, SourceId};
use heos_api::types::browse::{BroseSourceItem, BrowsableMedia};

use crate::views::browse::render_media_list_item;
use crate::views::pages::page;

#[derive(Debug)]
pub struct BrowseMusicContainerPage {
    pub source_id: SourceId,
    pub count: usize,
    pub returned: usize,
    pub container_id: ContainerId,
    pub items: Vec<BrowsableMedia>,
    pub range: Range,
}

impl BrowseMusicContainerPage {
    pub fn next_link(&self) -> Option<String> {
        if (self.count as u16) > self.range.start {
            let next = self.range.next();
            Some(format!(
                "/sources/{}/containers/{}?{}",
                &self.source_id,
                &self.container_id,
                next.as_query_str()
            ))
        } else {
            None
        }
    }

    pub fn prev_link(&self) -> Option<String> {
        self.range.previous().map(|next| {
            format!(
                "/sources/{}/containers/{}?{}",
                &self.source_id,
                &self.container_id,
                next.as_query_str()
            )
        })
    }

    pub fn render_html(&self) -> Markup {
        page(html!({
            nav {
                ol {
                    li { a href="/sources/" { ( "Back to sources")} }
                    li { a href=( format!("/sources/{}/browse", self.source_id)) { ( "Back to source")} }
                    li { a href=( format!("/sources/{}/containers/{}", self.source_id, self.container_id)) { ( "Back to parent container")} }
                }

            }
            ul .media-list {
                @for item in &self.items {
                    ( render_media_list_item(item, &self.source_id) )
                }
            }
            nav {
                ol {
                    @if let Some(link) = self.prev_link() {
                         li { { a href=(link) { ( "prev" ) } } }
                    }
                    @if let Some(next) = self.next_link() {
                        li { a href=(next) { ( "next" ) } }
                    }
                }
            }
        }))
    }
}

impl IntoResponse for BrowseMusicContainerPage {
    fn into_response(self) -> Response {
        self.render_html().into_response()
    }
}
