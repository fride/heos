use std::net::IpAddr;
use anyhow::Context;
use crate::error::HeosError;
use heos_api::types::player::{NowPlayingMedia, PlayerStepLevel, PlayState};
use heos_api::types::{Level, OnOrOff, PlayerId};
use crate::{HeosApi, HeosEvent, HeosResult};
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use crate::next::HeosCommand::Pause;
use heos_api::types::browse::MusicSource;

#[derive(Clone)]
pub struct SinglePlayer {
    pub player_id: PlayerId,
    pub name: String,
    pub address: IpAddr,
    pub volume: Level,
    pub now_playing: Option<NowPlayingMedia>
}

#[derive(Clone)]
pub enum Zone {
    SinglePlayer(SinglePlayer),
    PlayerGroup {
        leader: SinglePlayer,
        name: String,
        group_volume: Level,
        members: Vec<SinglePlayer>,
    },
}

#[derive(Clone)]
pub enum HeosCommand {
    Reload, // Refresh the entire state please ;)
    FetchPlayers,
    FetchZones,
    FetchMusicSources,
    Play(PlayerId),
    Pause(PlayerId),
    Stop(PlayerId),
    SetGroup {
        leader: PlayerId,
        members: Vec<PlayerId>, // set tp empty to delete group
    },
}

#[derive(Clone)]
pub enum HeosResponse {
    CommandSucceeded(String),
    ZonesChanged(Vec<Zone>),
    ZoneVolume(PlayerId, Level),
    PlayerVolume(PlayerId, Level),
    PlayState(PlayerId, PlayState),
    PlayerMute(PlayerId, OnOrOff),
    Sources(Vec<MusicSource>)

}

// cqrs ;)
pub type HeosDriver = (CommandChannel, broadcast::Receiver<HeosResponse>);

#[derive(Clone)]
pub struct CommandChannel(mpsc::Sender<(HeosCommand, oneshot::Sender<HeosResult<()>>)>);

impl CommandChannel {
    pub async fn execute_command(&self, command: HeosCommand) -> HeosResult<()>{
        let (a,b) = oneshot::channel();
        let command = (command, a);
        let _ = self.0.send(command).await;
        let _ = b.await
            .context("Failed to wait for command!")?;
        Ok(())
    }
}

pub async fn create_heos_driver(mut api: HeosApi) -> HeosResult<HeosDriver> {
    let (response_channel_send, mut response_channel_receive) = broadcast::channel(64);
    let (command_channel_send, command_channel_receive) = mpsc::channel(32);

    {
        let event_response_sender = response_channel_send.clone();
        let event_command_sender = command_channel_send.clone();
        let mut events = api.events().await?;
        tokio::spawn(async move {
            while let Some(event) = events.recv().await {
                let _ = match event {
                    HeosEvent::SourcesChanged => {
                        event_command_sender.send(HeosCommand::FetchMusicSources).await
                            .context("Failed")
                    }
                    HeosEvent::PlayersChanged => {
                        event_command_sender.send(HeosCommand::FetchPlayers).await
                            .context("Failed")
                    }
                    HeosEvent::GroupChanged => {
                        event_command_sender.send(HeosCommand::FetchZones).await
                            .context("Failed")
                    }
                    HeosEvent::PlayerStateChanged { player_id, state } => {
                        // non async but blocking!?
                        event_response_sender.send(HeosResponse::PlayState(player_id, state))
                            .context("Failed")
                    }
                    HeosEvent::PlayerNowPlayingChanged { .. } => {}
                    HeosEvent::PlayerNowPlayingProgress { .. } => {}
                    HeosEvent::PlayerPlaybackError { .. } => {}
                    HeosEvent::PlayerVolumeChanged { .. } => {}
                    HeosEvent::PlayerQueueChanged { .. } => {}
                    HeosEvent::PlayerRepeatModeChanged { .. } => {}
                    HeosEvent::PlayerShuffleModeChanged { .. } => {}
                    HeosEvent::GroupVolumeChanged { .. } => {}
                    HeosEvent::UserChanged { .. } => {}
                };
            }
        });
    }


    Ok((CommandChannel(command_channel_send), response_channel_receive))
}
