use std::collections::BTreeMap;

use heos_api::types::group::Group;
use heos_api::types::player::{HeosPlayer, NowPlayingMedia, PlayState};
use heos_api::types::{AlbumId, Level, MediaId, PlayerId, QueueId, SourceId};

pub struct Zone {
    pub name: String,
    pub id: PlayerId,
    pub volume: Level,
    pub members: BTreeMap<PlayerId, (String, Level)>,
    pub now_playing: NowPlaying,
    pub state: PlayState,
}

impl Zone {
    pub fn play_state_class(&self) -> &'static str {
        match self.state {
            PlayState::Play => "fa-solid fa-play",
            PlayState::Pause => "fa-solid fa-stop",
            PlayState::Stop => "fa-solid fa-pause",
        }
    }
    pub fn now_playing_image(&self) -> &str {
        match &self.now_playing {
            NowPlaying::Noting => "/assets/playing_nothing.png",
            NowPlaying::Station { image_url, .. } => &image_url,
            NowPlaying::Song { image_url, .. } => &image_url,
        }
    }
}

pub struct Zones(Vec<Zone>);
impl Zones {
    pub fn iter(&self) -> std::slice::Iter<'_, Zone> {
        self.0.iter()
    }
}

#[derive(Debug, Clone)]
pub enum NowPlaying {
    Noting,
    Station {
        song: String,
        album: String,
        artist: String,
        image_url: String,
        station: String,
        mid: MediaId,
        qid: QueueId,
        sid: SourceId,
        album_id: AlbumId,
    },
    Song {
        song: String,
        album: String,
        artist: String,
        image_url: String,
        mid: MediaId,
        qid: QueueId,
        sid: SourceId,
        album_id: AlbumId,
    },
}

impl NowPlaying {
    pub fn song(&self) -> &str {
        match &self {
            NowPlaying::Noting => "-",
            NowPlaying::Station { song, .. } => &song,
            NowPlaying::Song { song, .. } => &song,
        }
    }
    pub fn artist(&self) -> &str {
        match &self {
            NowPlaying::Noting => "-",
            NowPlaying::Station { artist, .. } => &artist,
            NowPlaying::Song { artist, .. } => &artist,
        }
    }
    pub fn album(&self) -> &str {
        match &self {
            NowPlaying::Noting => "-",
            NowPlaying::Station { album, .. } => &album,
            NowPlaying::Song { album, .. } => &album,
        }
    }
}

impl From<NowPlayingMedia> for NowPlaying {
    fn from(media: NowPlayingMedia) -> Self {
        match media.station {
            None => NowPlaying::Song {
                song: media.song,
                album: media.album,
                artist: media.artist,
                image_url: media.image_url,
                mid: media.mid,
                qid: media.qid,
                sid: media.sid,
                album_id: media.album_id,
            },
            Some(station) => NowPlaying::Station {
                song: media.song,
                album: media.album,
                artist: media.artist,
                image_url: media.image_url,
                station,
                mid: media.mid,
                qid: media.qid,
                sid: media.sid,
                album_id: media.album_id,
            },
        }
    }
}
impl From<(Vec<HeosPlayer>, Vec<Group>)> for Zones {
    fn from(input: (Vec<HeosPlayer>, Vec<Group>)) -> Self {
        let mut zones = Vec::new();
        let mut players: BTreeMap<PlayerId, HeosPlayer> = input
            .0
            .into_iter()
            .map(|player| (player.player_id, player))
            .collect();
        for group in input.1 {
            if let Some(leader) = players.remove(&group.gid) {
                let mut members: BTreeMap<PlayerId, (String, Level)> = group
                    .players
                    .iter()
                    .filter_map(|member| players.remove(&member.pid))
                    .map(|player| (player.player_id, (player.name, player.volume)))
                    .collect();
                let name: Vec<String> =
                    members
                        .values()
                        .fold(vec![leader.name.clone()], |mut acc, player| {
                            acc.push(player.0.clone());
                            acc
                        });
                members.insert(
                    leader.player_id,
                    (leader.name.clone(), leader.volume.clone()),
                );
                zones.push(Zone {
                    name: name.join(" + "),
                    id: leader.player_id,
                    volume: group.volume,
                    members,
                    now_playing: leader
                        .now_playing
                        .map(|m| m.into())
                        .unwrap_or(NowPlaying::Noting),
                    state: leader.play_state,
                });
            }
        }
        for (pid, player) in players {
            zones.push(Zone {
                name: player.name,
                id: pid,
                volume: player.volume,
                now_playing: player
                    .now_playing
                    .map(|m| m.into())
                    .unwrap_or(NowPlaying::Noting),
                members: Default::default(),
                state: player.play_state,
            })
        }
        Zones(zones)
    }
}
