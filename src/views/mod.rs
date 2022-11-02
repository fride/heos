use maud::{html, Markup, DOCTYPE};
use serde_json::to_string;

pub mod zone;

pub mod home;
pub mod sources;
/// A basic header with a dynamic `page_title`.
fn header(page_title: &str) -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1";
            link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/4.7.0/css/font-awesome.min.css";
            link rel="stylesheet" href="/static/style.css";
            script src="https://unpkg.com/htmx.org@1.8.2" integrity="sha384-+8ISc/waZcRdXCLxVgbsLzay31nCdyZXQxnsUy++HJzJliTzxKWr0m1cIEMyUzQu" crossorigin="anonymous" {}
            title { (page_title) }
        }
    }
}

/// The final Markup, including `header` and `footer`.
///
/// Additionally takes a `greeting_box` that's `Markup`, not `&str`.
pub fn page(title: &str, name: String, contents: Markup) -> Markup {
    let tabs = vec![
        ("Zones".to_string(), "/zones".to_string(), name == "Zones"),
        ("Quellen".to_string(), "/music_sources".to_string(), name == "Music Sources"),
    ];
    html! {
        (DOCTYPE)
        html {
            // Add the header markup to the page
            (header(title))
            body {
                div id="main" style="margin: 1em;"
                hx-swap="outerHtml" {
                    (render_tabs(tabs))
                    div class="tab-content" {
                        (contents)
                    }
                }
            }
        }
    }
}

pub fn render_tabs(tabs: Vec<(String,String, bool)>) -> Markup {
    html! {
        div class="tab-list" {
            @for (name, link, selected) in tabs {
                a.selected[selected]
                hx-get=(link)
                hx-swap="outerHtml"
                hx-target="#main"
                hx-select="#main"
                href=(link) { (name) }
            }
        }
    }
}
