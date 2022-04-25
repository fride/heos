use actix_web::{middleware, web, App, HttpResponse, HttpServer, Result};
use askama::Template;
use rusty_heos::Zone;

#[derive(Template)]
#[template(path = "zones.html")]
pub struct ZonesTemplate{
    pub zones: Vec<Zone>
}

mod filters {
    use std::fmt::Display;
    use askama::filters::format;

    pub fn optional<P : Display>(s: Option<P>) -> ::askama::Result<String> {
        Ok(s.map(|s|format!("{}",s)).unwrap_or("".to_owned()))
    }
}
