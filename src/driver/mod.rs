use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use druid::platform_menus::win::file::new;
use serde::{Deserialize, Serialize};
use crate::{ApiCommand, HeosApi, HeosResult};
use anyhow::{Context, Result};

use crate::model::group::GroupInfo;
use crate::model::Level;
use crate::model::player::{NowPlayingMedia, PlayerInfo};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Player {
    pub name: String,
    pub id: i64,
    pub volume: Option<Level>,
    pub now_playing: Option<NowPlayingMedia>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Group {
    pub name: String,
    pub id: i64,
    pub volume: Option<Level>,
    pub players: Vec<Player>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Players {
    pub ungrouped: Vec<Player>,
    pub grouped: Vec<Group>,
}

#[derive(Clone)]
pub struct HeosDriver{
    api: HeosApi,
    players: Arc<Mutex<Players>>
}

impl HeosDriver {
    pub async fn new(api: HeosApi) -> Self{
        Self { api,
            players : Arc::new(Mutex::new(Players::default()))}
    }

    pub fn get_players(&self) -> Players {
        let mutex_guard = self.players.lock().unwrap();
        let cloned = mutex_guard.clone();
        cloned
    }

    pub async fn init(&mut self) -> HeosResult<()>{
        println!("1");
        let (s,mut r) = tokio::sync::oneshot::channel();
        self.api.execute_command(ApiCommand::RegisterForChangeEvents(s)).await;
        // anyhow!?
        let mut events = r.await.map_err(|error|anyhow::Error::new(error))??;
        println!("2");
        //let api_clone = self.api.clone();
        tokio::spawn(async move {
            println!("Waiting for events");
            while let Some(event) = events.recv().await {
                println!("Got an event: {:?}", &event);
            }
        });
        println!("3");
        let new_players = self.api.get_players().await?;
        let new_groups = self.api.get_groups().await?;
        let mut players = self.players.lock().unwrap();
        *players = (new_players, new_groups).into();
        println!("4");
        Ok(())
    }
}


// impls ....
impl From<(Vec<PlayerInfo>, Vec<GroupInfo>)> for Players {
    fn from(source: (Vec<PlayerInfo>, Vec<GroupInfo>)) -> Self {
        let mut players: BTreeMap<i64, Player> = source
            .0
            .into_iter()
            .map(|p| (p.pid.clone(), p.into()))
            .collect();

        let grouped: Vec<Group> = source
            .1
            .into_iter()
            .map(|group_info| {
                let mut players = group_info
                    .players
                    .iter()
                    .filter_map(|group_info| players.remove(&group_info.pid))
                    .collect();
                Group {
                    name: group_info.name,
                    id: group_info.gid,
                    volume: None,
                    players,
                }
            })
            .collect();
        let ungrouped = players.into_values().collect();
        Self { grouped, ungrouped }
    }
}
impl From<PlayerInfo> for Player {
    fn from(source: PlayerInfo) -> Self {
        Player {
            name: source.name,
            id: source.pid,
            ..Default::default()
        }
    }
}
