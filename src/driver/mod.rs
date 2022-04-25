pub mod state;

use im::Vector;
use std::sync::{Arc, Mutex};
use log::error;
use serde::Serialize;

use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_stream::StreamExt;

use crate::connection::Connection;
use crate::driver::state::DriverState;
use crate::model::event::HeosEvent;
use crate::model::group::{GroupInfo, GroupVolume};
use crate::model::player::{PlayState, PlayerInfo, PlayerNowPlayingMedia, PlayerVolume};
use crate::model::{GroupId, Level, OnOrOff, PlayerId, Repeat};
use crate::{HeosError, HeosResult};

use crate::api::HeosApi;
pub use state::{Player, Zone};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ApiCommand {
    GetPlayers,
    GetGroups,
    RefreshState,
    LoadPlayerVolume(PlayerId),
    LoadGroupVolume(PlayerId),
    LoadNowPLaying(PlayerId),
}

pub enum ApiResults {
    Players(Vec<PlayerInfo>),
    Groups(Vec<GroupInfo>),
    PlayerVolumes(PlayerVolume),
    GroupVolumes(GroupVolume),
    PlayerNowPlaying(PlayerNowPlayingMedia),
    GroupVolumeChanged(GroupId, Level, OnOrOff),
    PlayerVolumeChanged(PlayerId, Level, OnOrOff),
    PlayerPlayStateChanged(PlayerId, PlayState),
    PlayerRepeatModeChanged(PlayerId, Repeat),
    Error(HeosError),
}

type Shared<T> = Arc<Mutex<T>>;

pub struct HeosDriver(Sender<ApiCommand>, Shared<DriverState>);

impl HeosDriver {
    pub async fn new(connection: Connection) -> HeosResult<Self> {
        setup(connection).await
    }
    pub async fn init(&self) {
        self.0.send(ApiCommand::RefreshState).await;
    }
    pub fn zones(&self) -> Vec<Zone> {
        let state = self.1.lock().unwrap();
        state.zone_iter().cloned().collect()
    }
}

async fn setup(mut connection: Connection) -> HeosResult<HeosDriver> {
    println!("Setting up");
    let event_connection = connection.try_clone().await?;
    let state = Arc::new(Mutex::new(DriverState::default()));

    let (command_send, command_rec) = mpsc::channel::<ApiCommand>(12);
    let (result_send, result_rec) = mpsc::channel::<ApiResults>(12);

    command_handler::create_command_handler(connection, command_rec, result_send.clone());
    event_hander::create_event_handler(event_connection, command_send.clone(), result_send.clone());
    state_handler::create_state_handler(state.clone(), result_rec);

    println!("All done");
    Ok(HeosDriver(command_send, state))
}

mod state_handler;


mod command_handler;

mod event_hander;
