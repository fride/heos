use crate::model::group::GroupInfo;
use crate::model::player::{PlayerInfo, Progress};
use crate::model::{Level, PlayerId};

pub enum StateUpdate {
    Initial,
    PlaybackError(String),
    Error(String),
    PlayersChanged(Vec<PlayerInfo>),
    GroupsChanged(Vec<GroupInfo>),
    PlayerVolumeChanged(PlayerId,Level),
    GroupVolumeChanged(PlayerId,Level),
    NowPlayingChanged(PlayerId,Level),
    NowPlayingProgressChanged(PlayerId, Progress)
}
