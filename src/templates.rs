use askama::Template;
use rusty_heos::model::zone::Zone;

#[derive(Template)]
#[template(path = "zones.html")]
pub struct ZonesTemplate {
    pub zones: Vec<Zone>,
}

mod filters {

    use std::fmt::Display;

    pub fn optional<P: Display>(s: Option<P>) -> ::askama::Result<String> {
        Ok(s.map(|s| format!("{}", s)).unwrap_or("".to_owned()))
    }
}
