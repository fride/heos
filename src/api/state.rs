use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::model::group::GroupInfo;
use crate::model::Level;
use crate::model::player::{NowPlayingMedia, PlayerInfo};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Player {
    pub name: String,
    pub id: i64,
    pub volume: Option<Level>,
    pub now_playing: Option<NowPlayingMedia>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Group {
    pub name: String,
    pub id: i64,
    pub volume: Option<Level>,
    pub players: Vec<Player>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Players {
    pub ungrouped: Vec<Player>,
    pub grouped: Vec<Group>,
}

impl Players {

}

impl From<PlayerInfo> for Player {
    fn from(source: PlayerInfo) -> Self {
        Player {
            name: source.name,
            id: source.pid,
            ..Default::default()
        }
    }
}

pub type HeosState = Arc<Mutex<Players>>;

fn extract_groups(players: Vec<PlayerInfo>, groups: Vec<GroupInfo>) -> (Vec<Group>, Vec<Player>) {
    let mut players: BTreeMap<i64, Player> = players
        .into_iter()
        .map(|p| (p.pid.clone(), p.into()))
        .collect();

    let grouped: Vec<Group> = groups
        .into_iter()
        .map(|group_info| {
            let mut players = group_info
                .players
                .iter()
                .filter_map(|group_info| players.remove(&group_info.pid))
                .collect();
            Group {
                name: group_info.name,
                id: group_info.gid,
                volume: None,
                players,
            }
        })
        .collect();
    (grouped, players.into_values().collect())
}

pub fn update(state: &mut HeosState, players: Vec<PlayerInfo>, groups: Vec<GroupInfo>) {
    let (grouped, ungrouped) = extract_groups(players, groups);
    let mut new_state = state.lock().unwrap();
    new_state.grouped = grouped;
    new_state.ungrouped = ungrouped
}
