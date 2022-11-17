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
            link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/4.7.0/css/font-awesome.min.css";
            link rel="stylesheet" type="text/css" href="/assets/style.css";
            script src="https://unpkg.com/htmx.org@1.8.2" integrity="sha384-+8ISc/waZcRdXCLxVgbsLzay31nCdyZXQxnsUy++HJzJliTzxKWr0m1cIEMyUzQu" crossorigin="anonymous" {}
        }
        body {
            (contents)
        }
    })
}

