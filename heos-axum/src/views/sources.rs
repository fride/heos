use crate::views::browse::render_media_list_item;
use crate::views::pages::page;
use axum::response::{IntoResponse, Response};
use heos_api::types::browse::{BroseSourceItem, MusicSource};
use heos_api::types::SourceId;
use maud::{html, Markup};
