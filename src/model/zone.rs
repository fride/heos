use std::fmt::Formatter;

use chrono::Duration;

use crate::model::{Level, Milliseconds, OnOrOff, PlayerId, Repeat, SourceId};
use crate::model::player::{MediaType, NowPlayingMedia, PlayerInfo, PlayState};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PlayingProgress {
    pub cur_pos: Milliseconds,
    pub duration: Option<Milliseconds>,
}

fn millis_to_str(millis: &Milliseconds) -> String {
    let duration = Duration::milliseconds(millis.clone() as i64); // overflow ;)
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60;
    let seconds = duration.num_seconds() % 60;
    if hours > 0 {
        format!("{}:{}:{}", hours, minutes, seconds)
    } else {
        format!("{}:{}", minutes, seconds)
    }
}

impl std::fmt::Display for PlayingProgress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.duration {
            None => write!(f, "{}", millis_to_str(&self.cur_pos)),
            Some(duration) => write!(
                f,
                "{} / {}",
                millis_to_str(&self.cur_pos),
                millis_to_str(&duration)
            ),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub volume: Option<Level>,
    pub now_playing: NowPlaying,
    pub progress: Option<PlayingProgress>,
    pub state: Option<PlayState>,
    pub repeat: Option<Repeat>,
    pub mute: Option<OnOrOff>,
    //pub last_seen: Option<DateTime<Utc>>
}
impl Player {}

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

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize, Clone)]
pub enum NowPlaying {
    Nothing,
    Station {
        station_name: String,
        album_id: String,
        album: String,
        artist: String,
        source_id: SourceId,
        image_url: String,
    },
    Song {
        song: String,
        album: String,
        artist: String,
        image_url: String,
        mid: String,
        album_id: String,
        source_id: SourceId,
    },
}
const EMPTY_STRING: String = String::new();

impl NowPlaying {
    pub fn image_url(&self) -> String {
        match self {
            NowPlaying::Nothing => EMPTY_STRING,
            NowPlaying::Station { image_url, .. } => image_url.clone(),
            NowPlaying::Song { image_url, .. } => image_url.clone(),
        }
    }
}
impl Default for NowPlaying {
    fn default() -> Self {
        NowPlaying::Nothing
    }
}

impl From<Option<NowPlayingMedia>> for NowPlaying {
    fn from(now: Option<NowPlayingMedia>) -> Self {
        now.map(|n| n.into()).unwrap_or(NowPlaying::Nothing)
    }
}

impl From<NowPlayingMedia> for NowPlaying {
    fn from(now: NowPlayingMedia) -> Self {
        tracing::info!("now playing:: {:?}", &now);
        match now.media_type {
            MediaType::Song => NowPlaying::Song {
                song: now.song.clone(),
                album: now.album,
                artist: now.artist,
                image_url: now.image_url,
                mid: now.mid,
                album_id: now.album_id,
                source_id: now.sid,
            },
            MediaType::Station => NowPlaying::Station {
                station_name: now.station.unwrap_or("".to_owned()),
                album_id: now.album_id,
                album: now.album,
                artist: now.artist,
                source_id: now.sid,
                image_url: now.image_url,
            },
        }
    }
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
    pub fn is_group(&self) -> bool {
        match self {
            Zone::SinglePlayer(_) => false,
            Zone::PlayerGroup { .. } => true,
        }
    }
    pub fn volume(&self) -> Option<Level> {
        match self {
            Zone::SinglePlayer(leader) => leader.volume.clone(),
            Zone::PlayerGroup { zone_volume, .. } => zone_volume.clone(),
        }
    }
    pub fn now_playing_media(&self) -> &NowPlaying {
        match self {
            Zone::SinglePlayer(leader) => &leader.now_playing,
            Zone::PlayerGroup { leader, .. } => &leader.now_playing,
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

    pub fn players(&self) -> Vec<&Player> {
        match self {
            Zone::SinglePlayer(leader) => vec![&leader],
            Zone::PlayerGroup {
                leader, members, ..
            } => {
                let mut players = vec![leader];
                players.extend(members);
                players
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
