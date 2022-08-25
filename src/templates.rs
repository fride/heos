use std::collections::BTreeMap;
use askama::Template;
use druid::Widget;

use rusty_heos::model::zone::NowPlaying;
use rusty_heos::model::zone::{Zone};

#[derive(Template)]
#[template(path = "zones.html")]
pub struct ZonesTemplate {
    pub zones: Vec<Zone>,
    pub player_names: BTreeMap<i64, String>,
}

impl ZonesTemplate {
    pub fn new<T: IntoIterator<Item = Zone>>(zones_to_use: T) -> Self {
        let zones: Vec<Zone> = zones_to_use.into_iter().collect();
        let mut player_names = BTreeMap::new();
        for zone in &zones {
            for player in &zone.players() {
                player_names.insert(player.id.clone(), player.name.clone());
            }
        }
        Self {
            zones,
            player_names,
        }
    }
}

mod filters {
    use std::fmt::Display;

    pub fn optional<P: Display>(s: Option<P>) -> ::askama::Result<String> {
        Ok(s.map(|s| format!("{}", s)).unwrap_or("".to_owned()))
    }
}

pub trait ZoneWidget {
    fn duration(&self) -> String;
}

impl ZoneWidget for Zone {
    fn duration(&self) -> String {
        let progress = match self {
            Zone::SinglePlayer(ref leader) => &leader.progress,
            Zone::PlayerGroup { ref leader, .. } => &leader.progress,
        };
        match progress {
            None => "".to_owned(),
            Some(progress) => format!("{}", progress),
        }
    }
}
