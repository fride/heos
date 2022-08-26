use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

use crate::connection::Connection;
use crate::driver::state::DriverState;
use crate::model::group::{GroupInfo, GroupVolume};
use crate::model::player::{PlayState, PlayerInfo, PlayerVolume};
use crate::model::zone::{NowPlaying, Player, Zone};
use crate::model::{GroupId, Level, Milliseconds, OnOrOff, PlayerId, Repeat};
use crate::{HeosError, HeosResult};

mod command_handler;
mod event_handler;
pub mod state;
mod state_handler;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ApiCommand {
    GetPlayers,
    GetGroups,
    RefreshState,
    LoadPlayerVolume(PlayerId),
    LoadGroupVolume(PlayerId),
    LoadNowPLaying(PlayerId),
}

#[derive(Debug)]
pub enum StateUpdates {
    Players(Vec<PlayerInfo>),
    Groups(Vec<GroupInfo>),
    PlayerVolumes(PlayerVolume),
    GroupVolumes(GroupVolume),
    PlayerNowPlaying(PlayerId, NowPlaying),
    GroupVolumeChanged(GroupId, Level, OnOrOff),
    PlayerVolumeChanged(PlayerId, Level, OnOrOff),
    PlayerPlayStateChanged(PlayerId, PlayState),
    PlayerRepeatModeChanged(PlayerId, Repeat),
    PlayerNowPlayingProgress {
        player_id: PlayerId,
        cur_pos: Milliseconds,
        duration: Option<Milliseconds>,
    },
    Error(HeosError),
}

type Shared<T> = Arc<Mutex<T>>;

pub struct HeosDriver(Sender<ApiCommand>, Shared<DriverState>);

impl HeosDriver {
    pub async fn new(connection: Connection) -> HeosResult<Self> {
        setup(connection).await
    }
    pub async fn init(&self) {
        let _ = self.0.send(ApiCommand::RefreshState).await;
    }

    pub fn zones(&self) -> Vec<Zone> {
        let mut all_zones = vec![];
        {
            let state = self.1.lock().unwrap();
            for zone in state.zone_iter() {
                all_zones.push(zone.clone())
            }
        }
        all_zones
    }

    pub fn players(&self) -> Vec<Player> {
        let mut players = vec![];
        {
            for zone in self.zones() {
                for player in zone.players() {
                    players.push(player.clone());
                }
            }
        }
        players
    }
}

async fn setup(mut connection: Connection) -> HeosResult<HeosDriver> {
    let event_connection = connection.try_clone().await?;
    let state = Arc::new(Mutex::new(DriverState::default()));

    let (command_send, command_rec) = mpsc::channel::<ApiCommand>(12);
    let (result_send, result_rec) = mpsc::channel::<StateUpdates>(12);

    command_handler::create_command_handler(connection, command_rec, result_send.clone());
    event_handler::create_event_handler(
        event_connection,
        command_send.clone(),
        result_send.clone(),
    );
    state_handler::create_state_handler(state.clone(), result_rec);
    Ok(HeosDriver(command_send, state))
}
