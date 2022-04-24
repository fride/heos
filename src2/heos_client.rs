use std::collections::BTreeMap;
use std::fmt::format;
use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tracing::event;

use crate::{HeosError, HeosResult, PlayerUpdate};
use crate::api::ApiCommand;
use crate::heos_client::state::StateMessage;
use crate::model::group::{GroupInfo, GroupVolume};
use crate::model::Level;
use crate::model::player::{NowPlayingMedia, PlayerInfo};

#[derive(Default, Clone)]
pub struct ClientState {
    players: BTreeMap<i64, PlayerInfo>,
    groups: BTreeMap<i64, GroupInfo>,
    player_volumes: BTreeMap<i64, Level>,
    group_volumes: BTreeMap<i64, Level>,
    last_error: Option<String>,
    last_command: Option<String>,
    now_playing: BTreeMap<i64, NowPlayingMedia>,
}

impl ClientState {
    pub fn handle(&mut self, event: PlayerUpdate) -> Vec<ApiCommand> {
        self.last_error = None;
        match event {
            PlayerUpdate::Players(players) => {
                // vec2map!(players, pid) TODO write this nonsense macro!
                self.players = players
                    .into_iter()
                    .map(|p| (p.pid.clone(), p))
                    .collect();
                self.players
                    .keys()
                    .map(|p| ApiCommand::GetNowPlaying(p.clone()))
                    .collect()
            },
            PlayerUpdate::Groups(groups) => {
                self.groups = groups.into_iter()
                    .map(|p| (p.gid.clone(), p))
                    .collect();
                vec![]
            }
            PlayerUpdate::NowPlaying(now) => {
                self.now_playing.insert(now.player_id, now.media);
                vec![]
            }
            PlayerUpdate::PlayerVolume(_, _) => {
                vec![]
            }
            PlayerUpdate::PlayingProgress(_, _, _) => {
                vec![]
            }
            PlayerUpdate::PlayerPlaybackError(_, _) => {
                vec![]
            }
            PlayerUpdate::PlayerVolumeChanged(_, _, _) => {
                vec![]
            }
            PlayerUpdate::MusicSources(_) => {
                vec![]
            }
        }
    }
}

pub struct HeosClient {
    state: mpsc::Sender<state::StateMessage>,
    api: mpsc::Sender<ApiCommand>,
}

impl HeosClient {
    pub fn new(api: mpsc::Sender<ApiCommand>,
               results: mpsc::Receiver<PlayerUpdate>,
               errors: mpsc::Receiver<HeosError>) -> Self {
        let api_clone = api.clone();
        let state = state::create_state_handler(api_clone);
        let state_clone = state.clone();

        let handle = {
            tokio::spawn(async {
                run(state_clone, results, errors).await;
            });
        };
        Self {
            state,
            api,
        }
    }
    pub async fn init(&self) {
        self.api.send(ApiCommand::GetPlayers).await.unwrap();
        self.api.send(ApiCommand::GetGroups).await.unwrap();
    }

    pub async fn get_players(&self) -> BTreeMap<i64, PlayerInfo> {
        let (s, r) = tokio::sync::oneshot::channel();
        let _ = self.state.try_send(state::StateMessage::Get(s));
        let state = r.await.unwrap();
        state.players
    }
}

async fn run(state: mpsc::Sender<state::StateMessage>,
             mut results: mpsc::Receiver<PlayerUpdate>,
             mut errors: mpsc::Receiver<HeosError>) {
    loop {
        print!("Waiting ...");
        tokio::select! {
            Some(response) = results.recv() => {
                println!("Got update");
                state.send(StateMessage::Set(response)).await;
            }
            Some(error) = errors.recv() => {
                println!("Got error");
                state.send(StateMessage::SetError(format!("{:?}",error)));
                // let mut state = state.lock().await;
                // state.last_error = Some(format!("{}", &error));
            }
        }
    }
}

mod state {
    use tokio::sync::mpsc;
    use tokio::sync::oneshot;

    use crate::api::ApiCommand;
    use crate::heos_client::ClientState;
    use crate::{HeosError, PlayerUpdate};

    pub enum StateMessage {
        Get(oneshot::Sender<ClientState>),
        Set(PlayerUpdate),
        SetError(String)
    }

    pub fn create_state_handler(api: mpsc::Sender<ApiCommand>) -> mpsc::Sender<StateMessage> {
        let (s, mut r) = mpsc::channel(20);
        tokio::spawn( async move {
            let mut state = ClientState::default();
            while let Some(msg) = r.recv().await {
                match msg {
                    StateMessage::Get(responder) => {
                        responder.send(state.clone());
                    },
                    StateMessage::Set(new_state) => {
                        let commands = state.handle(new_state);
                        for command in commands {
                            api.send(command).await.unwrap();
                        }
                    },
                    StateMessage::SetError(error) =>
                        state.last_error = Some(error)
                }
            }
        });
        s
    }
}
