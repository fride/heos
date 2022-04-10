use std::fmt;
use std::fmt::Formatter;

use crate::model::{Milliseconds, OnOrOff};
use crate::HeosError;

use super::common::Time;
use super::{AlbumId, GroupId, Level, PlayerId, QueueId};
use super::{MediaId, SourceId};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum PlayState {
    #[serde(rename = "play")]
    Play,
    #[serde(rename = "plause")]
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

impl std::str::FromStr for PlayState {
    type Err = HeosError;

    fn from_str(string: &str) -> Result<PlayState, Self::Err> {
        match string {
            "play" => Ok(PlayState::Play),
            "pause" => Ok(PlayState::Pause),
            "stop" => Ok(PlayState::Stop),
            c => Err(HeosError::ParserError {
                message: format!("can't convert {} to PlayState", c),
            }),
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct PlayerPlayState {
    pub player_id: PlayerId,
    pub state: PlayState,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct Progress {
    pub current_position: u64,
    pub duration_in_ms: u64,
}

impl Progress {
    pub fn new(current_position: u64, duration_in_ms: u64) -> Progress {
        Progress {
            current_position,
            duration_in_ms,
        }
    }
}

impl fmt::Display for Progress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let current_position: Time = self.current_position.into();
        if self.duration_in_ms == 0 {
            write!(f, "{}", current_position)
        } else {
            let duration_in_ms: Time = self.duration_in_ms.into();
            write!(f, "{} / {} ", current_position, duration_in_ms)
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
        let (pid, level) = t;
        PlayerVolume {
            player_id: pid,
            level: level,
        }
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct PlayerStepLevel {
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
    pub player_id: PlayerId,
    pub mode: super::PlayMode,
}

#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq)]
pub struct PlayerMute {
    pub player_id: i64,
    pub state: OnOrOff,
}

#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq)]
pub struct NowPlayingProgress {
    pub player_id: PlayerId,
    pub current_position: Milliseconds,
    pub duration_in_ms: Milliseconds,
}
