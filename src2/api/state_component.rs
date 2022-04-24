use std::collections::BTreeMap;

use tokio::sync::mpsc::{Receiver, Sender};

use tokio::sync::watch;
use crate::api::ApiCommand;

use crate::model::group::GroupInfo;
use crate::model::player::{PlayerInfo, Progress};
use crate::model::{Level, PlayerId};
use crate::PlayerUpdate;

#[derive(Debug, Clone, Deserialize, Serialize)]
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

//
//
// pub fn state_component(api: Sender<ApiCommand>, mut updates: Receiver<PlayerUpdate>) -> watch::Receiver<StateUpdate> {
//     let (tx, rx) = watch::channel(StateUpdate::Initial);
//     //
//     // question here. Use a lock and stuff like thisy.
//     tokio::spawn(async move {
//         while let Some(event) = updates.recv().await {
//             // TODO
//             match event {
//                 PlayerUpdate::Players(players) => {
//                     for pid in players.iter().map(|p|p.pid) {
//                         api.send(ApiCommand::GetNowPlaying(pid)).await;
//                     }
//                     tx.send(StateUpdate::PlayersChanged(players))
//                 }
//                 PlayerUpdate::Groups(groups) => {
//                     tx.send(StateUpdate::GroupsChanged(groups))
//                 }
//                 PlayerUpdate::NowPlaying(_) => {}
//                 PlayerUpdate::PlayerVolume(_, _) => {}
//                 PlayerUpdate::PlayingProgress(_, _, _) => {}
//                 PlayerUpdate::PlayerPlaybackError(_, _) => {}
//                 PlayerUpdate::PlayerVolumeChanged(_, _, _) => {}
//                 PlayerUpdate::MusicSources(_) => {}
//             };
//             tx.send(state.clone());
//         }
//     });
//     rx
// }
