use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};



use crate::contoller::Volume;
use crate::model::browse::MusicSource;
use crate::model::player::*;
use crate::model::PlayerId;
use crate::model::zone::NowPlaying;

type Shared<T> = Arc<Mutex<T>>;

#[derive(Debug, Clone, Default)]
pub struct State {
    user: Shared<Option<String>>,
    players: Shared<BTreeMap<PlayerId, PlayerInfo>>,
    player_states: Shared<BTreeMap<PlayerId, PlayState>>,
    player_volumes: Shared<BTreeMap<PlayerId, Volume>>,
    now_playing: Shared<BTreeMap<PlayerId, NowPlaying>>,
    music_sources: Shared<Vec<MusicSource>>,
}

impl State {
    pub fn set_music_sources<M: IntoIterator<Item = MusicSource>>(&self, sources: M) {
        let mut state = self.music_sources.lock().unwrap();
        state.clear();
        for source in sources {
            state.push(source);
        }
    }
    pub(crate) fn get_music_sources(&self) -> Vec<MusicSource> {
        let sources = self.music_sources.lock().unwrap();
        sources.clone()
    }

    pub(crate) fn set_player_state(&self, play_state: PlayerPlayState) {
        let mut state = self.player_states.lock().unwrap();
        state.insert(play_state.player_id, play_state.state);
    }

    pub fn set_player_volume(&self, pid: PlayerId, volume: Volume) {
        let mut state = self.player_volumes.lock().unwrap();
        state.insert(pid, volume);
    }

    pub fn set_now_playing(&self, pid: PlayerId, now: NowPlaying) {
        let mut state = self.now_playing.lock().unwrap();
        state.insert(pid, now);
    }

    pub fn set_players(&self, new_players: Vec<PlayerInfo>) {
        let mut players = self.players.lock().unwrap();
        players.clear();
        for player in new_players {
            players.insert(player.pid, player);
        }
    }
    pub fn get_players(&self) -> Vec<PlayerInfo> {
        let players = self.players.lock().unwrap();
        let res = players.values().cloned().collect();
        res
    }
}
