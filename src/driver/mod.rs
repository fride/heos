











use crate::{HeosApi, HeosResult};
use crate::connection::Connection;
use crate::driver::event_loop::start_event_loop;
use crate::driver::state::{DriverState, Player, Zone};
use crate::model::{PlayerId};




use crate::util::Shared;


mod state;
mod event_loop;
pub struct Driver {
    state: Shared<DriverState>,
    api: HeosApi
}

impl Driver {
    pub async fn create(mut connection: Connection) -> HeosResult<Self> {
        let command_connection = connection.try_clone().await?;
        let api = HeosApi::new(command_connection);
        let state = Shared::new(DriverState::default());
        let driver = Self {
            state: state.clone(),
            api: api.clone()
        };
        driver.init().await?;
        start_event_loop(api.clone(),  connection, state.clone());
        Ok(driver)
    }
    async fn init(&self) -> HeosResult<()> {
        let player_infos = self.api.get_player_infos().await?;
        for player in player_infos {
            let volume = self.api.get_volume(player.pid.clone()).await?;
            let play_state = self.api.get_play_state(player.pid.clone()).await?;
            self.state.with_mutable_state(|state| {
                state.add_player(player);
                state.player_volumes.insert(volume.player_id, volume.level);
                state.set_play_state(play_state.player_id, play_state.state);
            })
        }
        Ok(())
    }
    pub fn get_players(&self) -> Vec<Player>{
        let keys : Vec<PlayerId> = self.state.with_state(|s| s.players.keys().cloned().collect());
        println!("Found keys: {:?}", &keys );
        keys.into_iter()
            .map(|pid| Player::new(self.state.clone(), pid))
            .collect()
    }

    pub fn get_zones(&self) -> Vec<Zone> {
        let zones = self.state.with_state(|s| {
            s.grouped_player_ids()
        });
        zones.into_iter()
            .map(|(pid,members)| {
                Zone::new(pid, members, self.state.clone())
            }).collect()
    }
}



#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tokio::join;
    use super::*;

    pub type Shared<T> = Arc<Mutex<T>>;


    #[derive(Debug, Default)]
    pub struct Foo {
        players: BTreeMap<PlayerId, Shared<PlayerInfo>>,
    }
    #[tokio::test]
    pub async fn test_stuff() {
        let foo1 : Shared<Foo> = Arc::new(Mutex::new(Foo::default()));
        let foo2 = foo1.clone();
        let foo3 = foo1.clone();
        let h1 = tokio::spawn(async move {
            let new_player = PlayerInfo{
                name: "".to_string(),
                pid: 1,
                lineout: None,
                ip: None,
                model: None,
                network: None,
                version: None,
                gid: None,
                control: None
            };
            {
                let mut state = foo1.lock().unwrap();
                state.players.insert(new_player.pid.clone(), Arc::new(Mutex::new(new_player)));
            }
            tokio::time::sleep(Duration::from_millis(500)).await;

        });
        let h2 = tokio::spawn(async move {
            let new_player = PlayerInfo{
                name: "Player zwei ist super".to_string(),
                pid: 2,
                lineout: None,
                ip: None,
                model: None,
                network: None,
                version: None,
                gid: None,
                control: None
            };
            {
                let mut state = foo2.lock().unwrap();
                state.players.insert(new_player.pid.clone(), Arc::new(Mutex::new(new_player)));
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
            {
                let mut state = foo2.lock().unwrap();
                state.players.entry(2)
                    .and_modify(|player| {
                        let mut player = player.lock().unwrap();
                        player.gid = Some(-12);
                    });
            }
        });
        h1.await;
        h2.await;
        let state = foo3.lock().unwrap();
        for (_,x) in &state.players {
            println!("Player: {:?}", &x.lock().unwrap());
        }
    }
}
