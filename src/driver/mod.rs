use crate::connection::Connection;
use crate::driver::event_loop::start_event_loop;
pub use crate::driver::state::{DriverState, Player, Zone};
use crate::model::PlayerId;
use crate::{HeosApi, HeosResult};

use crate::util::Shared;

mod event_loop;
mod state;

pub struct Driver {
    state: Shared<DriverState>,
    api: HeosApi,
}

impl Driver {
    pub async fn create(mut connection: Connection) -> HeosResult<Self> {
        let command_connection = connection.try_clone().await?;
        let api = HeosApi::new(command_connection);
        let state = Shared::new(DriverState::default());
        let driver = Self {
            state: state.clone(),
            api: api.clone(),
        };
        driver.init().await?;
        start_event_loop(api.clone(), connection, state.clone());
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
    pub fn get_players(&self) -> Vec<Player> {
        let keys: Vec<PlayerId> = self
            .state
            .with_state(|s| s.players.keys().cloned().collect());
        println!("Found keys: {:?}", &keys);
        keys.into_iter()
            .map(|pid| Player::new(self.state.clone(), pid))
            .collect()
    }

    pub fn get_zones(&self) -> Vec<Zone> {
        let zones = self.state.with_state(|s| s.grouped_player_ids());
        zones
            .into_iter()
            .map(|(pid, members)| Zone::new(pid, members, self.state.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    pub async fn test_stuff() {
        unimplemented!()
    }
}
