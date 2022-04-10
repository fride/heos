use bytes::Buf;
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::{HeosError, HeosEvent, HeosResult, PlayerUpdate};
use crate::components::ApiCommand;

pub fn heos_event_aggregator_component(mut events: Receiver<HeosEvent>, api: Sender<ApiCommand>, updates: Sender<PlayerUpdate>, errors: Sender<HeosError>) {

    tokio::spawn(async move {
       while let Some(event) = events.recv().await {
            match event {
                HeosEvent::SourcesChanged => {
                    api.send(ApiCommand::GetMusicSources).await;
                }
                HeosEvent::PlayersChanged => {
                    api.send(ApiCommand::GetPlayers).await;
                }
                HeosEvent::GroupChanged => {
                    api.send(ApiCommand::GetGroups).await;
                    api.send(ApiCommand::GetPlayers).await;
                }
                HeosEvent::PlayerStateChanged { .. } => {}
                HeosEvent::PlayerNowPlayingChanged { player_id  } => {
                    api.send(ApiCommand::GetNowPlaying(player_id)).await;
                }
                HeosEvent::PlayerNowPlayingProgress { player_id, cur_pos, duration } => {
                    updates.send(PlayerUpdate::PlayingProgress(player_id, cur_pos, duration)).await;
                }
                HeosEvent::PlayerPlaybackError { player_id, error } => {
                    updates.send(PlayerUpdate::PlayerPlaybackError(player_id, error)).await;
                }
                HeosEvent::PlayerVolumeChanged { player_id, level ,mute} => {
                    updates.send(PlayerUpdate::PlayerVolumeChanged(player_id, level, mute)).await;
                }
                HeosEvent::PlayerQueueChanged { .. } => {
                    // todo -> load queue?
                }
                HeosEvent::PlayerRepeatModeChanged { player_id, repeat } => {
                    // TODO
                }
                HeosEvent::PlayerShuffleModeChanged { .. } => {}
                HeosEvent::GroupVolumeChanged { .. } => {}
                HeosEvent::UserChanged { .. } => {}
            }
       }
    });
}
