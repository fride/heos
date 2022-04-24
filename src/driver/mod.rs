pub mod state;

use im::Vector;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_stream::StreamExt;

use crate::connection::Connection;
use crate::driver::state::DriverState;
use crate::model::event::HeosEvent;
use crate::model::group::{GroupInfo, GroupVolume};
use crate::model::player::{PlayState, PlayerInfo, PlayerNowPlayingMedia, PlayerVolume};
use crate::model::{GroupId, Level, OnOrOff, PlayerId};
use crate::{HeosError, HeosResult};

use crate::api::HeosApi;
pub use state::{Player, Zone};

#[derive(Debug, Clone)]
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

    create_command_handler(connection, command_rec, result_send.clone());
    create_event_handler(event_connection, command_send.clone(), result_send.clone());
    create_state_handler(state.clone(), result_rec);

    println!("All done");
    Ok(HeosDriver(command_send, state))
}

fn create_state_handler(state: Shared<DriverState>, mut results: Receiver<ApiResults>) {
    tokio::spawn(async move {
        // TODO add timestamps and waiting indeicators. ;)
        while let Some(result) = results.recv().await {
            match result {
                ApiResults::Players(players) => {
                    let mut state = state.lock().unwrap();
                    state.set_players(players);
                }
                ApiResults::Groups(groups) => {
                    let mut state = state.lock().unwrap();
                    state.set_groups(groups);
                }
                ApiResults::PlayerVolumes(player_volume) => {
                    println!("Set Player volume");
                    let mut state = state.lock().unwrap();
                    state.update_player(player_volume.player_id.clone(), move |player| {
                        player.volume = Some(player_volume.level.clone());
                    })
                }
                ApiResults::GroupVolumes(_) => {}
                ApiResults::PlayerNowPlaying(player_now_playing) => {
                    println!("Setting now playin");
                    let mut state = state.lock().unwrap();
                    state.update_player(player_now_playing.player_id.clone(), move |player| {
                        player.now_playing = Some(player_now_playing.media.clone());
                    })
                }
                ApiResults::GroupVolumeChanged(_, _, _) => {}
                ApiResults::PlayerVolumeChanged(player_id, level, mute) => {
                    let mut state = state.lock().unwrap();
                    state.update_player(player_id.clone(), move |player| {
                        player.volume = Some(level.clone());
                        player.mute = Some(mute.clone());
                    })
                }
                ApiResults::PlayerPlayStateChanged(_, _) => {}
                ApiResults::Error(_) => {}
            }
        }
    });
}

async fn handle_command(
    command: ApiCommand,
    connection: &mut Connection,
    results: &mpsc::Sender<ApiResults>,
) {
    let response = match command {
        ApiCommand::GetPlayers => {
            let response = connection.load_players().await;
            response.map(|res| vec![ApiResults::Players(res)])
        }
        ApiCommand::GetGroups => {
            let response = connection.get_groups().await;
            response.map(|res| vec![ApiResults::Groups(res)])
        }
        ApiCommand::RefreshState => load_state(connection).await,
        ApiCommand::LoadPlayerVolume(pid) => connection
            .get_volume(pid)
            .await
            .map(|v| vec![ApiResults::PlayerVolumes(v)]),
        ApiCommand::LoadGroupVolume(gid) => {
            let volume = connection.get_group_volume(gid).await;
            volume.map(|v| vec![ApiResults::GroupVolumes(v)])
        }
        ApiCommand::LoadNowPLaying(pid) => connection
            .get_now_playing_media(pid)
            .await
            .map(|now| vec![ApiResults::PlayerNowPlaying(now)]),
    };
    match response {
        Ok(responses) => {
            for response in responses {
                results.send(response).await;
            }
        }
        Err(err) => {
            println!("Command failed! {:?}", &err);
            results.send(ApiResults::Error(err)).await;
        }
    }
}

async fn load_state(connection: &mut Connection) -> Result<Vec<ApiResults>, HeosError> {
    let mut responses = vec![];
    let players: Vec<PlayerInfo> = connection.load_players().await?;
    let groups: Vec<GroupInfo> = connection.get_groups().await?;
    let pids: Vector<PlayerId> = players.iter().map(|p| p.pid).collect();
    let gids: Vector<GroupId> = groups.iter().map(|p| p.gid).collect();
    responses.push(ApiResults::Players(players));
    responses.push(ApiResults::Groups(groups));

    for pid in &pids {
        let now_playing = connection.get_now_playing_media(pid.clone()).await?;
        responses.push(ApiResults::PlayerNowPlaying(now_playing));
        let player_volume = connection.get_volume(pid.clone()).await?;
        responses.push(ApiResults::PlayerVolumes(player_volume));
    }
    for gid in &gids {
        let group_volume = connection.get_group_volume(gid.clone()).await?;
        responses.push(ApiResults::GroupVolumes(group_volume));
    }
    Ok(responses)
}

pub fn create_command_handler(
    mut connection: Connection,
    mut commands: mpsc::Receiver<ApiCommand>,
    results: mpsc::Sender<ApiResults>,
) {
    println!("Setting up create_command_handler");
    tokio::spawn(async move {
        println!("Waiting for commands ");
        while let Some(command) = commands.recv().await {
            println!("Got command {:?}", &command);
            handle_command(command, &mut connection, &results).await;
        }
    });
}

pub fn create_event_handler(
    connection: Connection,
    commands: mpsc::Sender<ApiCommand>,
    results: mpsc::Sender<ApiResults>,
) {
    tokio::spawn(async move {
        let events = connection.into_event_streamm();
        tokio::pin!(events);
        while let Some(event) = events.next().await {
            match event {
                Err(_e) => {
                    println!("Error");
                }
                Ok(HeosEvent::PlayersChanged) => {
                    commands.send(ApiCommand::GetPlayers).await;
                }
                Ok(HeosEvent::SourcesChanged) => {}
                Ok(HeosEvent::GroupChanged) => {
                    commands.send(ApiCommand::GetGroups).await;
                }
                Ok(HeosEvent::PlayerStateChanged { player_id, state }) => {
                    results
                        .send(ApiResults::PlayerPlayStateChanged(player_id, state))
                        .await;
                }
                Ok(HeosEvent::PlayerNowPlayingChanged { player_id }) => {
                    commands.send(ApiCommand::LoadNowPLaying(player_id)).await;
                }
                Ok(HeosEvent::PlayerNowPlayingProgress { .. }) => {}
                Ok(HeosEvent::PlayerPlaybackError { .. }) => {}
                Ok(HeosEvent::PlayerVolumeChanged {
                    player_id,
                    level,
                    mute,
                }) => {
                    results
                        .send(ApiResults::GroupVolumeChanged(player_id, level, mute))
                        .await;
                }
                Ok(HeosEvent::PlayerQueueChanged { .. }) => {}
                Ok(HeosEvent::PlayerRepeatModeChanged { .. }) => {}
                Ok(HeosEvent::PlayerShuffleModeChanged { .. }) => {}
                Ok(HeosEvent::GroupVolumeChanged {
                    group_id,
                    level,
                    mute,
                }) => {
                    results
                        .send(ApiResults::GroupVolumeChanged(group_id, level, mute))
                        .await;
                }
                Ok(HeosEvent::UserChanged { .. }) => {}
            }
        }
    });
}
