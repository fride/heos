use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::contoller::Volume;
use crate::model::browse::MusicSource;
use crate::model::group::{GroupInfo, GroupRole, GroupVolume};
use crate::model::player::*;
use crate::model::zone::NowPlaying;
use crate::model::{GroupId, Level, Milliseconds, PlayerId};

pub type Shared<T> = Arc<Mutex<T>>;
pub fn shared<T>(value: T) -> Shared<T> {
    Arc::new(Mutex::new(value))
}

#[derive(Debug, Clone, Default)]
pub struct State {
    user: Shared<Option<String>>,
    players: Shared<BTreeMap<PlayerId, PlayerInfo>>,
    groups: Shared<BTreeMap<GroupId, GroupInfo>>,
    group_volumes: Shared<BTreeMap<GroupId, Level>>,
    player_states: Shared<BTreeMap<PlayerId, PlayState>>,
    player_volumes: Shared<BTreeMap<PlayerId, Volume>>,
    now_playing: Shared<BTreeMap<PlayerId, NowPlaying>>,
    now_playing_progress: Shared<BTreeMap<PlayerId, NowPlayingProgress>>,
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

    pub fn set_groups(&self, new_groups: Vec<GroupInfo>) {
        let mut players = self.players.lock().unwrap();
        let mut groups = self.groups.lock().unwrap();
        groups.clear();
        for group in new_groups {
            for member in &group.players {
                match &member.role {
                    GroupRole::Leader => players
                        .entry(member.pid)
                        .and_modify(|player| player.gid = Some(member.pid)),
                    GroupRole::Member => players
                        .entry(group.gid)
                        .and_modify(|player| player.gid = Some(member.pid)),
                };
            }
            groups.insert(group.gid, group);
        }
    }
    pub fn get_groups(&self) -> Vec<GroupInfo> {
        let groups = self.groups.lock().unwrap();
        let res = groups.values().cloned().collect();
        res
    }
    pub fn set_group_volume(&self, volume: GroupVolume) {
        let mut group_volumes = self.group_volumes.lock().unwrap();
        group_volumes.insert(volume.group_id, volume.level);
    }
    pub fn set_now_playing_progress(
        &self,
        player_id: PlayerId,
        cur_pos: Milliseconds,
        duration: Option<Milliseconds>,
    ) {
        let mut now_playing_progress = self.now_playing_progress.lock().unwrap();
        let mut entry = now_playing_progress
            .entry(player_id)
            .or_insert(NowPlayingProgress::default());
        entry.duration_in_ms = duration;
        entry.current_position = cur_pos;
    }
}
