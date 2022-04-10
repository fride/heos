use std::collections::BTreeMap;

use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::oneshot;
use tokio::sync::watch;
use tracing::debug;
use tracing_subscriber::fmt::format;

use crate::model::group::GroupInfo;
use crate::model::Level;
use crate::model::player::PlayerInfo;
use crate::PlayerUpdate;

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct State {
    pub players: BTreeMap<i64, PlayerInfo>,
    pub volumes: BTreeMap<i64, Level>,
    pub groups: BTreeMap<i64, GroupInfo>,
}

pub fn state_component(mut updates: Receiver<PlayerUpdate>) -> watch::Receiver<State>{
    let (tx, mut rx) = watch::channel(State::default());
    //
    // question here. Use a lock and stuff like thisy.
    tokio::spawn(async move {
        let mut state = State::default();
        while let Some(event) = updates.recv().await {
            // TODO
            match event {
                PlayerUpdate::Players(players) => {
                    state.players.clear();
                    state.players = players.iter().map(|p| (p.pid, p.clone())).collect(); // into_iter would be better!?
                }
                PlayerUpdate::Groups(mut groups) => {
                    state.groups = groups.into_iter().map(|g| (g.gid, g)).collect();
                }
                PlayerUpdate::NowPlaying(_) => {}
                PlayerUpdate::PlayerVolume(_, _) => {}
                PlayerUpdate::PlayingProgress(_, _, _) => {}
                PlayerUpdate::PlayerPlaybackError(_, _) => {}
                PlayerUpdate::PlayerVolumeChanged(_, _, _) => {}
                PlayerUpdate::MusicSources(_) => {}
            };
            tx.send(state.clone());
        }
    });
    rx
}
