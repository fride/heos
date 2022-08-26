use std::borrow::BorrowMut;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use tokio::sync::watch;
use crate::contoller::command::SetPlayState;
use crate::contoller::state::{Shared, shared};
use crate::model::player::{NowPlayingProgress, PlayerInfo, PlayState, Progress};
use crate::model::{GroupId, Level, PlayerId};
use crate::model::browse::MusicSource;
use crate::model::group::GroupInfo;
use crate::model::zone::NowPlaying;
use crate::spielwiese::CommandChannel;
use crate::Volume;

#[derive(Debug, Clone, Default)]
pub struct State {
    user: Option<String>,
    players: BTreeMap<PlayerId, PlayerInfo>,
    groups: BTreeMap<GroupId, GroupInfo>,
    group_volumes: BTreeMap<GroupId, Level>,
    player_states: BTreeMap<PlayerId, PlayState>,
    player_volumes: BTreeMap<PlayerId, Volume>,
    now_playing: BTreeMap<PlayerId, NowPlaying>,
    now_playing_progress: BTreeMap<PlayerId, NowPlayingProgress>,
    music_sources: Vec<MusicSource>

}

pub struct Player{
    player_id: PlayerId,
    name: Shared<String>,
    ip: Shared<String>,
    model: Shared<String>,
    network: Shared<String>,
    version: Shared<String>,
    gid: Shared<Option<GroupId>>,

    volume: Shared<Level>,

    command_channel: CommandChannel
}

impl Player {

    pub fn id(&self) -> PlayerId {
        self.player_id
    }

    pub fn name(&self) -> String {
        self.name.lock().unwrap().clone()
    }

    pub fn volume(&self) -> Level{
        self.volume.lock().unwrap().clone()
    }
    pub async fn set_play_state(&self, play_state: PlayState) {
        let command = SetPlayState { player_id: self.player_id, state: play_state };
        //let _ = self.command_channel.send(command).await;
    }
}


