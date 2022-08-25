use chrono::{DateTime, Utc};
use druid::widget::Controller;
use std::collections::BTreeMap;
use std::net::IpAddr;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::{Connection, HeosResult};
use crate::connection::CommandResult;
use crate::model::{PlayerId, SourceId};
use crate::v2::types::*;

pub struct HeosCommand {
    payload: String,
    result_handler: oneshot::Receiver<HeosResult<CommandResult>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: PlayerId,
    pub name: String,
    pub volume: Option<Level>,
    pub now_playing: NowPlaying,
    pub progress: NowPlayingProgress,
    pub state: Option<PlayState>,
    pub repeat: Option<Repeat>,
    pub mute: Option<OnOrOff>,

    #[serde(skip)]
    pub last_seen: Option<DateTime<Utc>>,
}
pub struct Player {
    state: Arc<Mutex<PlayerState>>,
    commands: mpsc::Sender<HeosCommand>
}

impl Player {
    pub fn new(state: Arc<Mutex<PlayerState>>,
               commands: mpsc::Sender<HeosCommand>) -> Self{
        Self {
            state,
            commands
        }
    }

    pub fn get_id(&self) -> PlayerId {
        self.state.lock().unwrap().id.clone()
    }

    pub fn get_name(&self) -> String {
        self.state.lock().unwrap().name.clone()
    }

    pub fn get_now_playing(&self) -> NowPlaying {
        self.state.lock().unwrap().now_playing.clone()
    }

    pub fn get_progress(&self) -> NowPlayingProgress {
        self.state.lock().unwrap().progress.clone()
    }
}

pub struct HeosController {
    players: Arc<Mutex<BTreeMap<PlayerId, PlayerState>>>

}
