use heos_api::types::player::{MediaType, NowPlayingMedia};

pub mod zones;
#[derive(Debug)]
pub enum TrackName {
    Song {
        artist: String,
        album: String,
        song: String,
    },
    Station {
        artist: String,
        album: String,
        song: String,
        station: String,
    },
}

impl TrackName {
    pub fn from_now_playing(now_playing: &NowPlayingMedia) -> Self {
        match now_playing.media_type {
            MediaType::Song => TrackName::Song {
                album: now_playing.album.clone(),
                artist: now_playing.artist.clone(),
                song: now_playing.song.clone(),
            },
            MediaType::Station => TrackName::Station {
                album: now_playing.album.clone(),
                artist: now_playing.artist.clone(),
                song: now_playing.song.clone(),
                station: now_playing.station.clone().unwrap_or("".to_string()),
            },
        }
    }
}

pub struct NowPlaying(NowPlayingMedia);
impl NowPlaying {
    pub fn name(&self) -> TrackName {
        TrackName::from_now_playing(&self.0)
    }
}
