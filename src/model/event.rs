use super::*;
use super::player::*;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum HeosEvent {
    #[serde(rename = "event/sources_changed")]
    SourcesChanged,

    #[serde(rename = "event/players_changed")]
    PlayersChanged,

    #[serde(rename = "event/groups_changed")]
    GroupChanged,

    #[serde(rename = "event/player_state_changed")]
    PlayerStateChanged {
        #[serde(rename = "pid")]
        player_id: PlayerId,
        state: PlayState,
    },

    #[serde(rename = "event/player_now_playing_changed")]
    PlayerNowPlayingChanged {
        #[serde(rename = "pid")]
        player_id: PlayerId,
    },

    #[serde(rename = "event/player_now_playing_progress")]
    PlayerNowPlayingProgress {
        #[serde(rename = "pid")]
        player_id: PlayerId,
        cur_pos: Milliseconds,
        duration: Option<Milliseconds>,
    },

    #[serde(rename = "event/player_playback_error")]
    PlayerPlaybackError {
        #[serde(rename = "pid")]
        player_id: PlayerId,
        error: String,
    },

    #[serde(rename = "event/player_volume_changed")]
    PlayerVolumeChanged {
        #[serde(rename = "pid")]
        player_id: PlayerId,
        level: Level,
        mute: OnOrOff,
    },

    #[serde(rename = "event/player_queue_changed")]
    PlayerQueueChanged {
        #[serde(rename = "pid")]
        player_id: PlayerId,
    },

    #[serde(rename = "event/repeat_mode_changed")]
    PlayerRepeatModeChanged {
        #[serde(rename = "pid")]
        player_id: PlayerId,
        repeat: Repeat,
    },

    #[serde(rename = "event/shuffle_mode_changed")]
    PlayerShuffleModeChanged {
        #[serde(rename = "pid")]
        player_id: PlayerId,
        shuffle: OnOrOff,
    },

    #[serde(rename = "event/group_volume_changed")]
    GroupVolumeChanged {
        #[serde(rename = "gid")]
        group_id: GroupId,
        level: Level,
        mute: OnOrOff,
    },

    #[serde(rename = "event/user_changed")]
    UserChanged {
        #[serde(rename = "un")]
        user_name: Option<String>,
    },
}
