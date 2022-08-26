
use crate::api::HeosApi;
use crate::contoller::command::SetPlayState;
use crate::contoller::command::{CommandChannel, InitController};
use crate::model::browse::MusicSource;
use crate::model::{Level, OnOrOff};
use crate::{Connection, HeosResult};
use state::*;
use tokio::sync::oneshot;

use crate::model::player::{PlayerInfo, PlayerPlayState};

#[derive(Debug, Clone)]
pub struct Volume {
    pub level: Level,
    pub mute: OnOrOff,
}
mod command;
mod event;
mod state;
mod state2;

pub mod command2;

#[derive(Debug)]
pub struct Controller {
    state: State,
    api: CommandChannel,
}

impl Controller {
    pub async fn new(mut connection: Connection) -> HeosResult<Self> {
        let state = State::default();
        let command_connection = connection.try_clone().await?;
        let api = CommandChannel::new(command_connection, state.clone());
        let _ = event::event_handler(api.clone(), connection, state.clone()).await;
        let controller = Self { state, api };
        controller.init().await;
        Ok(controller)
    }

    async fn init(&self) {
        let (s, r) = oneshot::channel();
        let _ = self.api.send_ack(InitController, s).await;
        tracing::info!("Init.");
        let _ = r.await;
        tracing::info!("Init done.");
    }

    pub fn get_players(&self) -> Vec<PlayerInfo> {
        self.state.get_players()
    }

    pub fn get_music_sources(&self) -> Vec<MusicSource> {
        self.state.get_music_sources()
    }

    pub async fn set_play_state(&self, state: PlayerPlayState) {
        let (s, r) = oneshot::channel();
        let _ = self
            .api
            .send_ack(
                SetPlayState {
                    state: state.state,
                    player_id: state.player_id,
                },
                s,
            )
            .await;
        r.await;
    }

    pub async fn stop_all(&self) {
        for player in self.state.get_players() {
            self.api.send(SetPlayState::stop(player.pid)).await;
        }
    }
}
