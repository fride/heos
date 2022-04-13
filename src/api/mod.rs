use std::mem::transmute;
use std::sync::{Arc, Mutex};

use async_stream::try_stream;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::sync::oneshot::{channel, Receiver};
use tokio_stream::Stream;
use tokio_stream::StreamExt;

use parsers::*;

use crate::{HeosError, HeosResult};
use crate::api::state::HeosState;
use crate::connection::*;
use crate::model::event::HeosEvent;
use crate::model::group::GroupInfo;
use crate::model::player::{NowPlayingMedia, PlayerInfo, PlayerPlayState, PlayerVolume, PlayState};
use crate::model::PlayerId;

mod parsers;
mod state;

pub type Responder<T> = oneshot::Sender<HeosResult<T>>;
pub type Listener<T> = oneshot::Receiver<HeosResult<T>>;

const GET_PLAYERS: &'static str = "player/get_players";
const GET_GROUPS: &'static str = "group/get_groups";

#[derive(Debug)]
pub enum ApiCommand {
    GetPlayers(Responder<Vec<PlayerInfo>>),
    GetPlayState(PlayerId, Responder<PlayerPlayState>),
    GetPlayerVolume(PlayerId, Responder<PlayerVolume>),
    GetNowPlayingMedia(PlayerId, Responder<NowPlayingMedia>),
    GetGroups(Responder<Vec<GroupInfo>>),
    RegisterForChangeEvents(Responder<mpsc::Receiver<HeosResult<HeosEvent>>>),
}
impl ApiCommand {
    pub fn get_player_volume(pid: PlayerId) -> (Listener<PlayerVolume>, ApiCommand) {
        let (s, mut r) = channel();
        (r, Self::GetPlayerVolume(pid, s))
    }
}
pub type HeosApiChannel = mpsc::Sender<ApiCommand>;

#[derive(Clone)]
pub struct HeosApi {
    channel: HeosApiChannel,
    state: state::HeosState
}

impl HeosApi {
    pub async fn connect(mut connection: crate::connection::Connection) -> HeosResult<Self> {
        let (s, mut r) = mpsc::channel(12);
        // spawn command execution in back ground
        let state: HeosState = Arc::new(Mutex::new(state::Players::default()));
        tokio::spawn(async move {
            let mut executor = CommandExecutor(connection);
            while let Some(cmd) = r.recv().await {
                let _ = executor.execute(cmd).await;
            }
        });
        Ok(Self { channel: s, state })
    }
    pub async fn execute_command(&self, command: ApiCommand) {
        self.channel.send(command).await;
    }

    pub async fn init(&self) -> HeosResult<()> {
        let players = self.get_players().await?;
        let groups = self.get_groups().await?;
        for player in &players {
            let play_state = self.get_play_state(player.pid.clone()).await?;
        }
        state::update(&mut self.state.clone(), players, groups);
        Ok(())
    }
    pub async fn get_players(&self) -> HeosResult<Vec<PlayerInfo>> {
        let (s, mut r) = oneshot::channel();
        let _ = self
            .channel
            .send(ApiCommand::GetPlayers(s))
            .await
            .expect("NUMM!");
        r.await.expect("BUMM!")
    }
    pub async fn get_play_state(&self, pid: PlayerId) -> HeosResult<PlayerPlayState> {
        let (s, mut r) = oneshot::channel();
        let _ = self
            .channel
            .send(ApiCommand::GetPlayState(pid, s))
            .await
            .expect("NUMM!");
        r.await.expect("BUMM!")
    }
    pub async fn get_groups(&self) -> HeosResult<Vec<GroupInfo>> {
        let (s, mut r) = oneshot::channel();
        let _ = self
            .channel
            .send(ApiCommand::GetGroups(s))
            .await
            .expect("NUMM!");
        r.await.expect("BUMM!")
    }
}

struct CommandExecutor(Connection);
impl CommandExecutor {
    pub async fn execute(&mut self, command: ApiCommand) {
        match command {
            ApiCommand::GetPlayers(responder) => {
                let response = self.execute_command(GET_PLAYERS).await;
                let _ = responder.send(response);
            }
            ApiCommand::GetPlayerVolume(pid, responder) => {
                let response = self
                    .execute_command(&format!("player/get_volume?pid={pid}", pid = pid))
                    .await;
                let _ = responder.send(response);
            }
            ApiCommand::GetPlayState(pid, responder) => {
                let command = format!("player/get_play_state?pid={pid}", pid = &pid);
                let response: HeosResult<PlayerPlayState> = self.execute_command(&command).await;
                let _ = responder.send(response);
            }
            ApiCommand::GetGroups(responder) => {
                let response = self.execute_command(GET_GROUPS).await;
                let _ = responder.send(response);
            },
            ApiCommand::RegisterForChangeEvents(responder) => {
                let mut events = self.0.try_clone().await.expect("Cant clone").into_event_streamm();
                let (event_tx, event_rx) = mpsc::channel(30);
                tokio::spawn(async move {
                    tokio::pin!(events);
                    while let Some(event) = events.next().await {
                        event_tx.send(event).await;
                    }
                });
                let _ = responder.send(Ok(event_rx));
            }
            _ => {}
        }
    }
    async fn execute_command<T>(&mut self, command: &str) -> HeosResult<T>
    where
        T: TryFrom<CommandResponse, Error = HeosError>,
    {
        println!("Sending: {}", command);
        let _ = self.0.write_frame(&command).await?;
        let response = self.0.read_command_response().await?;
        response.try_into()
    }
}
