use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use rusty_heos::model::group::GroupInfo;
use rusty_heos::model::Level;
use rusty_heos::model::player::{NowPlayingMedia, PlayerInfo};

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

impl From<(Vec<PlayerInfo>, Vec<GroupInfo>)> for Players {
    fn from(source: (Vec<PlayerInfo>, Vec<GroupInfo>)) -> Self {
        let mut players: BTreeMap<i64, Player> = source
            .0
            .into_iter()
            .map(|p| (p.pid.clone(), p.into()))
            .collect();

        let grouped: Vec<Group> = source
            .1
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
        let ungrouped = players.into_values().collect();
        Self { grouped, ungrouped }
    }
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
