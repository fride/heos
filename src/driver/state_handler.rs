use crate::driver::state::DriverState;
use crate::driver::{StateUpdates, Shared};
use crate::model::group::GroupVolume;
use tokio::sync::mpsc::Receiver;
use crate::model::player::NowPlayingProgress;
use crate::model::zone::PlayingProgress;

pub fn create_state_handler(state: Shared<DriverState>, mut results: Receiver<StateUpdates>) {
    tokio::spawn(async move {
        // TODO add timestamps and waiting indeicators. ;)
        while let Some(result) = results.recv().await {
            handle_result(&state, result);
        }
    });
}

fn handle_result(state: &Shared<DriverState>, result: StateUpdates) {
    match result {
        StateUpdates::Players(players) => {
            let mut state = state.lock().unwrap();
            state.set_players(players);
        }
        StateUpdates::Groups(groups) => {
            let mut state = state.lock().unwrap();
            state.set_groups(groups);
        }
        StateUpdates::PlayerVolumes(player_volume) => {
            let mut state = state.lock().unwrap();
            state.update_player(player_volume.player_id.clone(), move |player| {
                player.volume = Some(player_volume.level.clone());
            })
        }
        StateUpdates::GroupVolumes(group_volume) => {
            let mut state = state.lock().unwrap();
            state.set_group_volume(group_volume);
        }
        StateUpdates::PlayerNowPlaying(player_id, player_now_playing) => {
            println!("Setting now playin");
            let mut state = state.lock().unwrap();
            state.update_player(player_id.clone(), move |player| {
                player.now_playing = player_now_playing.clone()
            })
        }
        StateUpdates::GroupVolumeChanged(group_id, level, _mute) => {
            let mut state = state.lock().unwrap();
            state.set_group_volume(GroupVolume { group_id, level });
        }
        StateUpdates::PlayerVolumeChanged(player_id, level, mute) => {
            let mut state = state.lock().unwrap();
            state.update_player(player_id.clone(), move |player| {
                player.volume = Some(level.clone());
                player.mute = Some(mute.clone());
            })
        }
        StateUpdates::PlayerPlayStateChanged(player_id, play_state) => {
            let mut state = state.lock().unwrap();
            state.update_player(player_id.clone(), move |player| {
                player.state = Some(play_state);
            })
        }
        StateUpdates::Error(error) => {
            let mut state = state.lock().unwrap();
            state.set_error(error);
        }
        StateUpdates::PlayerRepeatModeChanged(player_d, repeat_mode) => {
            let mut state = state.lock().unwrap();
            state.update_player(player_d, move |player| {
                player.repeat = Some(repeat_mode.clone());
            });
        }
        StateUpdates::PlayerNowPlayingProgress { player_id, cur_pos, duration } => {
            let mut state = state.lock().unwrap();
            state.update_player(player_id, move |player| {
                player.progress = Some(PlayingProgress{
                    cur_pos: cur_pos.clone(),
                    duration: duration.clone()
                });
            });
        }
    }
}
