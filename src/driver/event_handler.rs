use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use crate::{Connection, HeosResult};
use crate::driver::{ApiCommand, StateUpdates};
use crate::model::event::HeosEvent;

fn event_to_results(event: HeosResult<HeosEvent>) -> (Vec<ApiCommand>, Vec<StateUpdates>) {
    match event {
        Err(err) => (vec![], vec![StateUpdates::Error(err)]),
        Ok(HeosEvent::PlayersChanged) => (vec![ApiCommand::GetPlayers], vec![]),
        Ok(HeosEvent::SourcesChanged) => (vec![], vec![]),
        Ok(HeosEvent::GroupChanged) => (vec![ApiCommand::GetGroups], vec![]),
        Ok(HeosEvent::PlayerStateChanged { player_id, state }) => (
            vec![],
            vec![StateUpdates::PlayerPlayStateChanged(player_id, state)],
        ),
        Ok(HeosEvent::PlayerNowPlayingChanged { player_id }) => {
            (vec![ApiCommand::LoadNowPLaying(player_id)], vec![])
        }
        Ok(HeosEvent::PlayerNowPlayingProgress { player_id, cur_pos, duration }) =>
            (vec![], vec![StateUpdates::PlayerNowPlayingProgress { player_id, cur_pos, duration }]),
        Ok(HeosEvent::PlayerPlaybackError {
               player_id: _,
               error: _,
           }) => (vec![], vec![]),
        Ok(HeosEvent::PlayerVolumeChanged {
               player_id,
               level,
               mute,
           }) => (
            vec![],
            vec![StateUpdates::PlayerVolumeChanged(player_id, level, mute)],
        ),
        Ok(HeosEvent::PlayerQueueChanged { .. }) => (vec![], vec![]),
        Ok(HeosEvent::PlayerRepeatModeChanged { player_id, repeat }) => (
            vec![],
            vec![StateUpdates::PlayerRepeatModeChanged(player_id, repeat)],
        ),
        Ok(HeosEvent::PlayerShuffleModeChanged { .. }) => (vec![], vec![]),
        Ok(HeosEvent::GroupVolumeChanged {
            group_id,
            level,
            mute,
        }) => (
            vec![],
            vec![StateUpdates::GroupVolumeChanged(group_id, level, mute)],
        ),
        Ok(HeosEvent::UserChanged { .. }) => (vec![], vec![]),
    }
}

pub fn create_event_handler(
    connection: Connection,
    commands: mpsc::Sender<ApiCommand>,
    results: mpsc::Sender<StateUpdates>,
) {
    tokio::spawn(async move {
        let events = connection.into_event_stream();
        tokio::pin!(events);
        while let Some(event) = events.next().await {
            let (commands_to_send, results_to_send) = event_to_results(event);
            for command_to_send in commands_to_send {
                commands
                    .send(command_to_send)
                    .await
                    .expect("failed to send command");
            }
            for result_to_send in results_to_send {
                results
                    .send(result_to_send)
                    .await
                    .expect("failed to send result");
            }
        }
    });
}
