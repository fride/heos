use heos_api::types::group::{Group, GroupRole};
use heos_api::types::player::{HeosPlayer, NowPlayingMedia};
use heos_api::types::{Level, PlayerId};
use heos_api::HeosDriver;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ZoneMember {
    pub id: PlayerId,
    pub name: String,
    pub volume: Level,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Zone {
    pub leader: HeosPlayer,
    pub members: Vec<ZoneMember>,
    pub group_volume: Option<Level>,
}

impl Into<Zone> for HeosPlayer {
    fn into(self) -> Zone {
        Zone {
            leader: self,
            members: vec![],
            group_volume: None,
        }
    }
}

impl Into<ZoneMember> for HeosPlayer {
    fn into(self) -> ZoneMember {
        ZoneMember {
            id: self.player_id,
            name: self.name,
            volume: self.volume,
        }
    }
}

impl Zone {
    pub fn name(&self) -> String {
        if self.members.is_empty() {
            self.leader.name.clone()
        } else {
            self.members
                .iter()
                .fold(self.leader.name.clone(), |acc, member| {
                    format!("{} + {}", acc, &member.name)
                })
        }
    }
    pub fn zone_id(&self) -> String {
        format!("zone{}", self.leader.player_id)
    }

    pub fn id(&self) -> PlayerId {
        self.leader.player_id
    }

    pub fn now_playing(&self) -> &Option<NowPlayingMedia> {
        &self.leader.now_playing
    }
    pub fn contains_member(&self, pid: PlayerId) -> bool {
        self.members.iter().find(|p| p.id == pid).is_some()
    }
}

impl Zone {
    pub fn get_zones(driver: &HeosDriver) -> Vec<Zone> {
        let players = driver.players();
        let groups = driver.groups();
        Zone::from_players_and_groups(players, groups)
    }

    fn from_players_and_groups<
        A: IntoIterator<Item = HeosPlayer>,
        B: IntoIterator<Item = Group>,
    >(
        players: A,
        groups: B,
    ) -> Vec<Zone> {
        let mut zones = vec![];
        let mut players: BTreeMap<PlayerId, HeosPlayer> = players
            .into_iter()
            .map(|player| (player.player_id, player))
            .collect();

        for group in groups {
            if let Some(leader) = group
                .leader()
                .and_then(|leader| players.remove(&leader.pid))
            {
                let members: Vec<HeosPlayer> = group
                    .players
                    .iter()
                    .filter_map(|member| {
                        if member.role == GroupRole::Leader {
                            None
                        } else {
                            players.remove(&member.pid)
                        }
                    })
                    .collect();
                zones.push(Zone {
                    leader,
                    members: members.into_iter().map(|p| p.into()).collect(),
                    group_volume: None,
                })
            }
        }
        for (_, player) in players {
            zones.push(player.into());
        }
        zones
    }
}

#[cfg(test)]
mod tests {

    use pretty_env_logger::env_logger;

    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    pub async fn zones() {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
        let driver = HeosDriver::new("192.168.178.35:1255").await.unwrap();
        driver.init().await.unwrap();

        let zones = Zone::get_zones(&driver);
        println!("{:?}", driver.players());
        println!("{:?}", driver.groups());
        println!("{:?}", zones);
    }
}
