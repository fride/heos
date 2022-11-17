use std::fmt;
use std::fmt::{Display, Formatter};

use crate::types::{Milliseconds, OnOrOff, PlayMode, Repeat};

use super::Time;
use super::{AlbumId, GroupId, Level, PlayerId, QueueId};
use super::{MediaId, SourceId};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum PlayState {
    #[serde(rename = "play")]
    Play,
    #[serde(rename = "pause")]
    Pause,
    #[serde(rename = "stop")]
    Stop,
}

impl fmt::Display for PlayState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                &PlayState::Play => "play",
                &PlayState::Pause => "pause",
                &PlayState::Stop => "stop",
            }
        )
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct PlayerPlayState {
    #[serde(alias = "pid")]
    pub player_id: PlayerId,
    pub state: PlayState,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct Progress {
    pub current_position: u64,
    pub duration_in_ms: Option<u64>,
}

impl Progress {
    pub fn new(current_position: u64, duration_in_ms: Option<u64>) -> Progress {
        Progress {
            current_position,
            duration_in_ms,
        }
    }
}

impl fmt::Display for Progress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let current_position: Time = self.current_position.into();
        match self.duration_in_ms {
            None => write!(f, "{}", current_position),
            Some(duration) => write!(f, "{} / {} ", current_position, duration),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PlayerInfo {
    pub name: String,
    pub pid: PlayerId,
    pub lineout: Option<u64>,
    pub ip: Option<String>,
    pub model: Option<String>,
    pub network: Option<String>,
    pub version: Option<String>,
    pub gid: Option<GroupId>,
    pub control: Option<String>,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum MediaType {
    #[serde(rename = "song")]
    Song,
    #[serde(rename = "station")]
    Station,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct NowPlayingMedia {
    #[serde(rename = "type")]
    pub media_type: MediaType,
    pub song: String,
    pub album: String,
    pub artist: String,
    pub image_url: String,
    pub station: Option<String>,
    pub mid: MediaId,
    pub qid: QueueId,
    pub sid: SourceId,
    pub album_id: AlbumId,
}

// needed as guess what! The request responds with an empty object
// instead of null ;)
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct NowPlayingMediaResponse {
    media: Option<NowPlayingMedia>,
}
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct PlayerNowPlayingMedia {
    #[serde(rename = "pid")]
    pub player_id: i64,
    pub media: NowPlayingMedia,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct PlayerVolume {
    #[serde(rename = "pid")]
    pub player_id: PlayerId,
    pub level: Level,
}

impl From<(PlayerId, Level)> for PlayerVolume {
    fn from(t: (PlayerId, Level)) -> Self {
        let (player_id, level) = t;
        PlayerVolume { player_id, level }
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct PlayerStepLevel {
    #[serde(rename = "pid")]
    pub player_id: PlayerId,
    pub step: u8,
}

#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq)]
pub struct QueueEntry {
    pub song: String,
    pub album: String,
    pub artist: String,
    pub image_url: String,
    pub qid: i64,
    pub mid: String,
    pub album_id: String,
}

#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq)]
pub struct PlayerPlayMode {
    #[serde(rename = "pid")]
    pub player_id: PlayerId,
    #[serde(flatten)]
    pub mode: super::PlayMode,
}

#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq)]
pub struct PlayerMute {
    #[serde(rename = "pid")]
    pub player_id: i64,
    pub state: OnOrOff,
}

#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq, Default)]
pub struct NowPlayingProgress {
    #[serde(rename = "pid")]
    pub player_id: PlayerId,
    pub current_position: Milliseconds,
    pub duration_in_ms: Option<Milliseconds>,
}

#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq)]
pub struct PlayerRepeatMode {
    #[serde(rename = "pid")]
    pub player_id: PlayerId,
    pub repeat: Repeat,
}

#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq)]
pub struct PlayerShuffleMode {
    #[serde(rename = "pid")]
    pub player_id: PlayerId,
    pub shuffle: OnOrOff,
}

#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq)]
pub struct PlayerPlaybackError {
    #[serde(rename = "pid")]
    pub player_id: PlayerId,
    pub error: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeosPlayer {
    pub player_id: PlayerId,
    pub name: String,
    pub volume: Level,
    pub now_playing: Option<NowPlayingMedia>,
    pub play_state: PlayState,
    pub in_group: Option<PlayerId>,
    pub mode: Option<PlayMode>,
}

impl HeosPlayer {
    pub fn is_single_player(&self) -> bool {
        self.in_group.is_none()
    }
    pub fn is_leader(&self) -> bool {
        self.in_group.filter(|gid| *gid == self.player_id).is_some()
    }
    pub fn as_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}
use serde_json::Value;

impl Display for HeosPlayer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        serde_json::to_string_pretty(self)
            .map_err(|_| std::fmt::Error)
            .and_then(|s| write!(f, "{}", s))
    }
}
