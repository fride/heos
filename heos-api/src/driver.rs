use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use tokio::net::ToSocketAddrs;
use tracing::debug;

use crate::types::browse::{BroseSourceItem, BrowseMusicContainerResponse, MusicSource};
use crate::types::event::HeosEvent;
use crate::types::group::{Group, GroupRole};
use crate::types::player::{HeosPlayer, PlayerInfo, QueueEntry};
use crate::types::system::AccountState;
use crate::types::{ContainerId, GroupId, PlayerId, Range, SourceId};
use crate::{HeosApi, HeosError, HeosResult};

#[derive(Default, Debug)]
struct DriverState {
    pub players: BTreeMap<PlayerId, HeosPlayer>,
    pub groups: BTreeMap<GroupId, Group>,
    pub music_sources: BTreeMap<SourceId, MusicSource>,
}

impl DriverState {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(DriverState::default()))
    }
}

#[derive(Clone)]
pub struct HeosDriver {
    api: HeosApi,
    state: Arc<Mutex<DriverState>>,
}

impl HeosDriver {
    pub async fn new<T: ToSocketAddrs>(addr: T) -> HeosResult<Self> {
        let api = HeosApi::connect(addr).await?;
        let state = DriverState::new();

        let driver = Self { api, state };
        let _ = driver.init().await;
        let _ = driver.start_event_listener().await;
        Ok(driver)
    }

    pub async fn init(&self) -> HeosResult<()> {
        let players = load_players(&self.api).await?;
        let groups = load_groups(&self.api).await?;
        let music_sources = self.api.get_music_sources().await?;
        {
            debug!(
                "Found {} players and {} groups",
                players.len(),
                groups.len()
            );
            println!(
                "Found {} players and {} groups",
                players.len(),
                groups.len()
            );
            let mut state = self.state.lock().unwrap();
            state.players.clear();
            state.groups.clear();
            state.players = players.into_iter().map(|p| (p.player_id, p)).collect();
            state.groups = groups.into_iter().map(|g| (g.gid, g)).collect();
            state.music_sources = music_sources.into_iter().map(|g| (g.sid, g)).collect();
        }
        Ok(())
    }
    pub async fn login(&self, un: String, pw: String) -> HeosResult<AccountState> {
        self.api.login(un, pw).await
    }

    pub fn players(&self) -> Vec<HeosPlayer> {
        let state = self.state.lock().unwrap();
        let players = state.players.values().cloned().collect();
        players
    }

    pub fn groups(&self) -> Vec<Group> {
        let state = self.state.lock().unwrap();
        let groups = state.groups.values().cloned().collect();
        groups
    }

    pub fn music_sources(&self) -> Vec<MusicSource> {
        let state = self.state.lock().unwrap();
        let music_sources = state.music_sources.values().cloned().collect();
        music_sources
    }

    pub async fn get_player_queue(
        &self,
        pid: PlayerId,
        range: Range,
    ) -> HeosResult<Vec<QueueEntry>> {
        self.api.get_queue(pid, range).await
    }

    // TODO this is a bit slow as the event will come anyways ....

    pub async fn create_group<C: IntoIterator<Item = PlayerId>>(
        &self,
        leader: PlayerId,
        members: C,
    ) -> HeosResult<()> {
        let members: BTreeSet<PlayerId> = members.into_iter().collect();
        // check if we do a valid request first
        if let Some(group) = self.groups().iter().find(|g| g.gid == leader) {
            let members_in_group: BTreeSet<PlayerId> = group
                .players
                .iter()
                .filter_map(|p| {
                    if p.role == GroupRole::Member {
                        Some(p.pid.clone())
                    } else {
                        None
                    }
                })
                .collect();
            if members == members_in_group {
                return Ok(());
            }
        }
        let mut group = vec![leader];
        group.extend(members);
        let _ = self.api.set_group(group).await?;
        let groups = load_groups(&self.api).await?;
        let mut state = self.state.lock().unwrap();
        state.groups = groups.into_iter().map(|g| (g.gid, g)).collect();
        Ok(())
    }

    pub async fn browse(&self, sid: SourceId) -> HeosResult<Vec<BroseSourceItem>> {
        self.api.browse_music_sources(sid).await
    }
    pub async fn browse_music_containers(
        &self,
        sid: &SourceId,
        container_id: &ContainerId,
        range: &Range,
    ) -> HeosResult<BrowseMusicContainerResponse> {
        self.api
            .browse_music_containers(sid, container_id, range)
            .await
    }

    async fn start_event_listener(&self) -> HeosResult<()> {
        let mut events = self.api.events().await?;
        let event_api = self.api.clone();
        let state = self.state.clone();
        tokio::spawn(async move {
            while let Some(event) = events.recv().await {
                let _ = HeosDriver::handle_event(event, &event_api, &state).await;
            }
        });
        Ok(())
    }

    async fn handle_event(
        event: HeosEvent,
        connection: &HeosApi,
        driver_state: &Arc<Mutex<DriverState>>,
    ) -> HeosResult<()> {
        match event {
            HeosEvent::SourcesChanged => {
                let _ = connection.get_music_sources().await.map(|sources| {
                    let mut state = driver_state.lock().unwrap();
                    state.music_sources = sources.into_iter().map(|s| (s.sid, s)).collect();
                });
            }
            HeosEvent::PlayersChanged => {
                let _ = load_players(&connection).await.map(|players| {
                    let mut state = driver_state.lock().unwrap();
                    state.players = players.into_iter().map(|s| (s.player_id, s)).collect();
                });
            }
            HeosEvent::GroupChanged => {
                let _ = load_groups(connection).await.map(|groups| {
                    let mut state = driver_state.lock().unwrap();
                    state.groups = groups.into_iter().map(|s| (s.gid, s)).collect();
                });
                let _ = load_players(connection).await.map(|players| {
                    let mut state = driver_state.lock().unwrap();
                    state.players = players.into_iter().map(|p| (p.player_id, p)).collect();
                });
            }
            HeosEvent::PlayerStateChanged { player_id, state } => {
                let mut driver_state = driver_state.lock().unwrap();
                if let Some(player) = driver_state.players.get_mut(&player_id) {
                    player.play_state = state
                }
            }
            HeosEvent::PlayerNowPlayingChanged { player_id } => {
                let _ =
                    connection
                        .get_now_playing_media(&player_id)
                        .await
                        .map(|now_playing_media| {
                            let mut state = driver_state.lock().unwrap();
                            if let Some(player) = state.players.get_mut(&player_id) {
                                player.now_playing = now_playing_media
                            }
                        });
            }
            HeosEvent::PlayerNowPlayingProgress { .. } => {}
            HeosEvent::PlayerPlaybackError { .. } => {}
            HeosEvent::PlayerVolumeChanged { .. } => {}
            HeosEvent::PlayerQueueChanged { .. } => {}
            HeosEvent::PlayerRepeatModeChanged { .. } => {}
            HeosEvent::PlayerShuffleModeChanged { .. } => {}
            HeosEvent::GroupVolumeChanged { .. } => {}
            HeosEvent::UserChanged { .. } => {}
        };
        Ok(())
    }
}

pub async fn load_groups(channel: &HeosApi) -> HeosResult<Vec<Group>> {
    let mut groups = vec![];
    let group_infos = channel.get_groups().await?;
    for group_info in group_infos {
        let volume = channel.get_group_volume(group_info.gid).await?;
        groups.push(Group {
            name: group_info.name,
            gid: group_info.gid,
            volume: volume.level,
            players: group_info.players,
        });
    }
    Ok(groups)
}

pub async fn load_players(channel: &HeosApi) -> Result<Vec<HeosPlayer>, HeosError> {
    let mut players = vec![];
    let player_infos = channel.get_player_infos().await?;
    for info in player_infos {
        players.push(fetch_player(channel, info).await?)
    }
    Ok(players)
}

async fn fetch_player(channel: &HeosApi, info: PlayerInfo) -> HeosResult<HeosPlayer> {
    let volume = channel.get_volume(&info.pid).await?.level;
    let state = channel.get_play_state(&info.pid).await?.state;
    let now_playing = channel.get_now_playing_media(&info.pid).await?;
    let mode = Some(channel.get_play_mode(&info.pid).await?.mode);

    Ok(HeosPlayer {
        player_id: info.pid,
        name: info.name,
        volume,
        now_playing,
        mode,
        play_state: state,
        in_group: info.gid,
    })
}
