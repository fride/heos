






use tokio::sync::{oneshot};


use state::*;

use crate::{Connection, HeosResult};
use crate::api::HeosApi;

use crate::contoller::command::{
    CommandChannel, InitController,
};
use crate::model::{Level, OnOrOff};
use crate::model::browse::MusicSource;

use crate::model::player::{
    PlayerInfo,
};


#[derive(Debug, Clone)]
pub struct Volume {
    pub level: Level,
    pub mute: OnOrOff,
}

mod command;

mod state;
mod event;

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
        Ok(Self { state, api })
    }
    pub async fn init(&mut self) {
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

    pub async fn stop_all(&self) {
        for _player in self.state.get_players() {}
    }
}
