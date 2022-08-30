use std::sync::{Arc, RwLock};

use crate::model::event::HeosEvent;
use crate::HeosApi;

type Shared<A> = Arc<RwLock<A>>;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum PlayerConnectionState {
    Connected,
    Disconnected,
}
// #[derive(Debug, Clone)]
// pub struct Player {
//     connection_state: PlayerConnectionState,
//     player_id: PlayerId,
//     player_info: Shared<PlayerInfo>,
//     play_state: Shared<PlayState>,
// }
//
// impl Player {
//     pub fn set_state(&self, state: PlayState) {
//         let state = self.play_state.write().unwrap();
//         state.
//     }
// }

pub async fn hande_event(event: HeosEvent, api: HeosApi) {
    match event {
        HeosEvent::SourcesChanged => {}
        HeosEvent::PlayersChanged => {}
        HeosEvent::GroupChanged => {}
        HeosEvent::PlayerStateChanged {
            player_id: _,
            state: _,
        } => {
            //.. PlayerPlayState {player_id, state}
        }
        HeosEvent::PlayerNowPlayingChanged { player_id } => {
            let _now_playing = api.get_now_playing_media(player_id).await;
        }
        HeosEvent::PlayerNowPlayingProgress {
            player_id: _,
            cur_pos: _,
            duration: _,
        } => {
            // new data typ!
        }
        HeosEvent::PlayerPlaybackError {
            player_id: _,
            error: _,
        } => {}
        HeosEvent::PlayerVolumeChanged {
            player_id: _,
            level: _,
            mute: _,
        } => {}
        HeosEvent::PlayerQueueChanged { player_id: _ } => {
            // api.get_queue(player_id)
        }
        HeosEvent::PlayerRepeatModeChanged {
            player_id: _,
            repeat: _,
        } => {}
        HeosEvent::PlayerShuffleModeChanged {
            player_id: _,
            shuffle: _,
        } => {}
        HeosEvent::GroupVolumeChanged {
            group_id: _,
            level: _,
            mute: _,
        } => {}
        HeosEvent::UserChanged { user_name: _ } => {}
    }
}
