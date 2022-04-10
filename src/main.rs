#[macro_use]
extern crate serde_derive;
// extern crate futures;
extern crate serde_json;
extern crate serde_qs as qs;

pub use error::{HeosError, HeosErrorCode};

use crate::components::PlayerUpdate;
use crate::model::event::HeosEvent;

pub mod connection;
mod error;
pub mod model;
pub(crate) mod parsers;

mod spielwiese;
pub type HeosResult<T> = Result<T, HeosError>;

pub mod components;

#[tokio::main]
async fn main() -> crate::HeosResult<()> {
    println!("Hello, world!");

    let connection = connection::Connection::connect("192.168.178.27:1255").await?;

    let (api, mut results, mut errors) = components::heos_components(connection).await?;

    api.send(components::ApiCommand::GetPlayers).await.unwrap();
    api.send(components::ApiCommand::GetGroups).await.unwrap();

    loop {
        tokio::select! {
            Some(response) = results.recv() => {
                //println!("{:?}", &response);
                match response {
                    PlayerUpdate::Players(players) => {
                        for p in players {
                            api.send(components::ApiCommand::GetNowPlaying(p.pid)).await;
                        }
                    }
                    PlayerUpdate::NowPlaying(_) => {}
                    _ => {}
                }
            }
            Some(error) = errors.recv() => {
                println!("Got Error: {}", error);
            }
        };
    }
}

mod foo {

    use tokio::sync::mpsc;
    use tokio::sync::oneshot;

    use crate::model::group::GroupInfo;
    use crate::model::player::{NowPlayingMedia, NowPlayingProgress, PlayState, PlayerInfo};
    use crate::model::PlayerId;

    use super::*;

    pub enum HeosCommand {
        LoadSources,
        LoadPlayers,
        LoadGroups,
        LoadNowPlayingMedia(PlayerId),
    }

    pub enum ModelUpdate {
        SetPlayers(Vec<PlayerInfo>),
        SetGroups(Vec<GroupInfo>),
        SetPlayState(PlayerId, PlayState),
        SetNowPlaying(PlayerId, NowPlayingMedia),
        SetNowPlayingProgress(NowPlayingProgress),
    }

    async fn event_handler(
        event: HeosEvent,
        model_updates: mpsc::Sender<ModelUpdate>,
        command_channel: mpsc::Sender<HeosCommand>,
    ) {
        match event {
            HeosEvent::SourcesChanged => {
                command_channel.send(HeosCommand::LoadSources).await;
            }
            HeosEvent::PlayersChanged => {
                command_channel.send(HeosCommand::LoadPlayers).await;
            }
            HeosEvent::GroupChanged => {
                command_channel.send(HeosCommand::LoadGroups).await;
            }
            HeosEvent::PlayerStateChanged { player_id, state } => {
                model_updates
                    .send(ModelUpdate::SetPlayState(player_id, state))
                    .await;
            }
            HeosEvent::PlayerNowPlayingChanged { player_id } => {
                command_channel
                    .send(HeosCommand::LoadNowPlayingMedia(player_id))
                    .await;
            }
            HeosEvent::PlayerNowPlayingProgress {
                player_id,
                cur_pos,
                duration,
            } => {
                model_updates
                    .send(ModelUpdate::SetNowPlayingProgress(NowPlayingProgress {
                        player_id,
                        current_position: cur_pos,
                        duration_in_ms: duration.unwrap(),
                    }))
                    .await;
            }
            HeosEvent::PlayerPlaybackError { .. } => {}
            HeosEvent::PlayerVolumeChanged { .. } => {}
            HeosEvent::PlayerQueueChanged { .. } => {}
            HeosEvent::PlayerRepeatModeChanged { .. } => {}
            HeosEvent::PlayerShuffleModeChanged { .. } => {}
            HeosEvent::GroupVolumeChanged { .. } => {}
            HeosEvent::UserChanged { .. } => {}
        }
    }

    enum ResponseChannel<T> {
        Mpsc(mpsc::Sender<HeosResult<T>>),
        OneShot(oneshot::Sender<T>),
    }

    enum FooBarCommand {
        GetPlayers(mpsc::Sender<HeosResult<Vec<PlayerInfo>>>),
    }
}
