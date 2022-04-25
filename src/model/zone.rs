use crate::model::player::{NowPlayingMedia, PlayState, PlayerInfo};
use crate::model::{Level, OnOrOff, PlayerId, Repeat};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub volume: Option<Level>,
    pub now_playing: Option<NowPlayingMedia>,
    pub state: Option<PlayState>,
    pub repeat: Option<Repeat>,
    pub mute: Option<OnOrOff>,
    //pub last_seen: Option<DateTime<Utc>>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Zone {
    SinglePlayer(Player),
    PlayerGroup {
        leader: Player,
        zone_name: String,
        zone_volume: Option<Level>,
        members: Vec<Player>,
        //last_seen: Option<DateTime<Utc>> // o
    },
}

impl Zone {
    pub fn single_player<P: Into<Player>>(player: P) -> Self {
        Zone::SinglePlayer(player.into())
    }

    pub fn group(leader: Player, members: Vec<Player>) -> Self {
        let zone_name = Zone::zone_name(&leader, &members);
        Zone::PlayerGroup {
            leader,
            zone_name,
            zone_volume: None,
            members,
        }
    }

    pub fn name(&self) -> String {
        match self {
            Zone::SinglePlayer(leader) => leader.name.clone(),
            Zone::PlayerGroup { zone_name, .. } => zone_name.clone(),
        }
    }
    pub fn set_volume(&mut self, level: Level) {
        match self {
            Zone::PlayerGroup { zone_volume, .. } => {
                *zone_volume = Some(level);
            }
            _ => {}
        }
    }
    pub fn volume(&self) -> Option<Level> {
        match self {
            Zone::SinglePlayer(leader) => leader.volume.clone(),
            Zone::PlayerGroup { zone_volume, .. } => zone_volume.clone(),
        }
    }
    pub fn now_playing_media(&self) -> Option<NowPlayingMedia> {
        match self {
            Zone::SinglePlayer(leader) => leader.now_playing.clone(),
            Zone::PlayerGroup { leader, .. } => leader.now_playing.clone(),
        }
    }

    pub fn id(&self) -> i64 {
        match self {
            Zone::SinglePlayer(sp) => sp.id.clone(),
            Zone::PlayerGroup { leader, .. } => leader.id.clone(),
        }
    }
    // todo - this is a bit silly ;)
    pub fn zone_name(leader: &Player, members: &Vec<Player>) -> String {
        format!(
            "{} + {}",
            &leader.name,
            members
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<&str>>()
                .join(" + ")
        )
    }
    pub fn find_player(&self, player_id: PlayerId) -> Option<&Player> {
        match self {
            Zone::SinglePlayer(ref player) => Some(player),
            Zone::PlayerGroup {
                leader, members, ..
            } => {
                if leader.id == player_id {
                    Some(&leader)
                } else {
                    for member in members {
                        if member.id == player_id {
                            return Some(&member);
                        }
                    }
                    None
                }
            }
        }
    }

    pub fn into_players(self) -> Vec<Player> {
        self.into()
    }

    pub fn with_players<A: IntoIterator<Item = Player>>(self, players: A) -> Self {
        match self {
            Zone::SinglePlayer(leader) => {
                let members = players.into_iter().collect();
                let zone_name = Zone::zone_name(&leader, &members);
                Zone::PlayerGroup {
                    leader,
                    zone_volume: None,
                    zone_name,
                    members,
                }
            }
            Zone::PlayerGroup {
                leader,
                zone_name: _zone_name,
                zone_volume,
                mut members,
            } => {
                let new_members = {
                    members.extend(players);
                    members
                };
                let zone_name = Zone::zone_name(&leader, &new_members);
                Zone::PlayerGroup {
                    leader,
                    zone_volume,
                    zone_name,
                    members: new_members,
                }
            }
        }
    }
}

impl Into<Player> for PlayerInfo {
    fn into(self) -> Player {
        Player {
            name: self.name,
            id: self.pid,
            ..Default::default()
        }
    }
}

impl Into<Zone> for PlayerInfo {
    fn into(self) -> Zone {
        match self.gid {
            None => Zone::SinglePlayer(self.into()),
            Some(gid) if gid == self.pid => {
                let leader = self.into();
                let zone_name = Zone::zone_name(&leader, &vec![]);
                Zone::PlayerGroup {
                    leader,
                    zone_volume: None,
                    zone_name,
                    members: vec![],
                }
            }
            Some(gid) => {
                let members = vec![self.into()];
                let leader = Player {
                    id: gid,
                    ..Default::default()
                };
                let zone_name = Zone::zone_name(&leader, &vec![]);
                Zone::PlayerGroup {
                    leader,
                    zone_volume: None,
                    zone_name,
                    members,
                }
            }
        }
    }
}

impl Into<Vec<Player>> for Zone {
    fn into(self) -> Vec<Player> {
        match self {
            Zone::SinglePlayer(player) => vec![player],
            Zone::PlayerGroup {
                leader,
                zone_volume: _,
                mut members,
                ..
            } => {
                members.insert(0, leader);
                members
            }
        }
    }
}
