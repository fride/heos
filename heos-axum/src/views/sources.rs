use axum::response::{IntoResponse, Response};
use maud::{html, Markup};

use heos_api::types::browse::{BroseSourceItem, MusicSource};
use heos_api::types::SourceId;

use crate::views::browse::render_media_list_item;
use crate::views::pages::page;

