use crate::{Connection, HeosResult};
use crate::contoller::command::{CommandChannel, GetPlayers};
use crate::contoller::state::State;
use crate::contoller::Volume;
use crate::model::event::HeosEvent;
use crate::model::player::PlayerPlayState;
use tokio_stream::StreamExt;

pub async fn event_handler(command_channel: CommandChannel,
                           connection: Connection,
                           state: State) {

    tokio::spawn(async move {
        let events = connection.into_event_stream();
        tokio::pin!(events);
        while let Some(event) = events.next().await {
            match event {
                Ok(event) => {
                    handle_event(event, &command_channel, &state).await;
                }
                Err(err) => {
                    println!("error in event fetching: {:?}", err);
                }
            }

        }
    });
}

pub async fn handle_event(event: HeosEvent,
                          command_channel: &CommandChannel,
                          state: &State) {
    match event {
        HeosEvent::SourcesChanged => {}
        HeosEvent::PlayersChanged => {
            let _ = command_channel.send(GetPlayers).await;
        }
        HeosEvent::GroupChanged => {

        }
        HeosEvent::PlayerStateChanged { player_id, state: play_state } => {
            state.set_player_state(PlayerPlayState{player_id, state: play_state});
        }
        HeosEvent::PlayerNowPlayingChanged { player_id } => {

        }
        HeosEvent::PlayerNowPlayingProgress { player_id, cur_pos,duration } => {

            //player_progress.insert(player_id, Progress{current_position: cur_pos, duration_in_ms: duration.unwrap_or_default()});
        }
        HeosEvent::PlayerPlaybackError { .. } => {}
        HeosEvent::PlayerVolumeChanged { player_id, level, mute } => {
            state.set_player_volume(player_id, Volume { level, mute});
        }
        HeosEvent::PlayerQueueChanged { .. } => {}
        HeosEvent::PlayerRepeatModeChanged { .. } => {}
        HeosEvent::PlayerShuffleModeChanged { .. } => {}
        HeosEvent::GroupVolumeChanged { .. } => {}
        HeosEvent::UserChanged { .. } => {}
    }
}
