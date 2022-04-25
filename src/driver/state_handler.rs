use im::Vector;
use crate::{Connection, HeosError};
use crate::driver::{ApiResults, Shared};
use crate::driver::state::DriverState;
use crate::model::group::{GroupInfo, GroupVolume};
use crate::model::player::PlayerInfo;
use crate::model::{GroupId, PlayerId};
use crate::api::HeosApi;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

pub fn create_state_handler(state: Shared<DriverState>, mut results: Receiver<ApiResults>) {
    tokio::spawn(async move {
        // TODO add timestamps and waiting indeicators. ;)
        while let Some(result) = results.recv().await {
            handle_result(&state, result);
        }
    });
}

fn handle_result(state: &Shared<DriverState>, result: ApiResults) {
    match result {
        ApiResults::Players(players) => {
            let mut state = state.lock().unwrap();
            state.set_players(players);
        }
        ApiResults::Groups(groups) => {
            let mut state = state.lock().unwrap();
            state.set_groups(groups);
        }
        ApiResults::PlayerVolumes(player_volume) => {
            println!("Set Player volume");
            let mut state = state.lock().unwrap();
            state.update_player(player_volume.player_id.clone(), move |player| {
                player.volume = Some(player_volume.level.clone());
            })
        }
        ApiResults::GroupVolumes(group_volume) => {
            let mut state = state.lock().unwrap();
            state.set_group_volume(group_volume);
        }
        ApiResults::PlayerNowPlaying(player_now_playing) => {
            println!("Setting now playin");
            let mut state = state.lock().unwrap();
            state.update_player(player_now_playing.player_id.clone(), move |player| {
                player.now_playing = Some(player_now_playing.media.clone());
            })
        }
        ApiResults::GroupVolumeChanged(group_id, level, mute) => {
            let mut state = state.lock().unwrap();
            state.set_group_volume(GroupVolume { group_id, level});
        }
        ApiResults::PlayerVolumeChanged(player_id, level, mute) => {
            let mut state = state.lock().unwrap();
            state.update_player(player_id.clone(), move |player| {
                player.volume = Some(level.clone());
                player.mute = Some(mute.clone());
            })
        }
        ApiResults::PlayerPlayStateChanged(player_id, play_state) => {
            let mut state = state.lock().unwrap();
            state.update_player(player_id.clone(), move |player| {
                player.state = Some(play_state);
            })
        }
        ApiResults::Error(error) => {
            let mut state = state.lock().unwrap();
            state.set_error(error);
        }
        ApiResults::PlayerRepeatModeChanged(player_d, repeat_mode) => {
            let mut state = state.lock().unwrap();
            state.update_player(player_d, move |player| {
                player.repeat = Some(repeat_mode.clone());
            });
        }
    }
}
