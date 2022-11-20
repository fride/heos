use crate::templates::statics::*;
use maud::{html, Markup, DOCTYPE};
pub mod music_containers;
pub mod music_sources;

pub fn page(contents: Markup) -> Markup {
    html!( {
        (DOCTYPE)
        head {
            meta charset="utf-8";
            title { ("H E O S") }
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1";
            script src="https://kit.fontawesome.com/17b53da5d5.js" crossorigin="anonymous" {}
            link rel="stylesheet" type="text/css" href=(format!("/assets/{}", style_css.name));
            script src="https://unpkg.com/htmx.org@1.8.2" integrity="sha384-+8ISc/waZcRdXCLxVgbsLzay31nCdyZXQxnsUy++HJzJliTzxKWr0m1cIEMyUzQu" crossorigin="anonymous" {}
        }
        body {
            main .main #main{
                (contents)
            }

        }
    })
}
