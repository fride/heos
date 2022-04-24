use tokio::sync::mpsc::{Receiver, Sender};

use crate::api::ApiCommand;
use crate::{HeosError, HeosEvent, PlayerUpdate};

pub fn heos_event_aggregator_component(
    mut events: Receiver<HeosEvent>,
    api: Sender<ApiCommand>,
    updates: Sender<PlayerUpdate>,
    _errors: Sender<HeosError>,
) {
    tokio::spawn(async move {
        while let Some(event) = events.recv().await {
            match event {
                HeosEvent::SourcesChanged => {
                    api.send(ApiCommand::GetMusicSources).await.unwrap();
                }
                HeosEvent::PlayersChanged => {
                    api.send(ApiCommand::GetPlayers).await.unwrap();
                }
                HeosEvent::GroupChanged => {
                    api.send(ApiCommand::GetGroups).await.unwrap();
                    api.send(ApiCommand::GetPlayers).await.unwrap();
                }
                HeosEvent::PlayerStateChanged { .. } => {}
                HeosEvent::PlayerNowPlayingChanged { player_id } => {
                    api.send(ApiCommand::GetNowPlaying(player_id))
                        .await
                        .unwrap();
                }
                HeosEvent::PlayerNowPlayingProgress {
                    player_id,
                    cur_pos,
                    duration,
                } => {
                    updates
                        .send(PlayerUpdate::PlayingProgress(player_id, cur_pos, duration))
                        .await
                        .unwrap();
                }
                HeosEvent::PlayerPlaybackError { player_id, error } => {
                    updates
                        .send(PlayerUpdate::PlayerPlaybackError(player_id, error))
                        .await
                        .unwrap();
                }
                HeosEvent::PlayerVolumeChanged {
                    player_id,
                    level,
                    mute,
                } => {
                    updates
                        .send(PlayerUpdate::PlayerVolumeChanged(player_id, level, mute))
                        .await
                        .unwrap();
                }
                HeosEvent::PlayerQueueChanged { .. } => {
                    // todo -> load queue?
                }
                HeosEvent::PlayerRepeatModeChanged {
                    player_id: _,
                    repeat: _,
                } => {
                    // TODO
                }
                HeosEvent::PlayerShuffleModeChanged { .. } => {}
                HeosEvent::GroupVolumeChanged { .. } => {}
                HeosEvent::UserChanged { .. } => {}
            }
        }
    });
}
