use log::trace;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Receiver;

use crate::{Connection, HeosResult};
use crate::api::HeosApi;
use crate::contoller::{State, Volume};
use crate::contoller::command::{ApiCommand, GetMusicSources};
use crate::model::OnOrOff::On;
use crate::model::player::PlayerInfo;

#[derive(Debug, Default)]
pub struct GetPlayers;

impl GetPlayers {
    pub async fn apply(self, connection: &mut Connection, state: &State) -> HeosResult<()> {
        tracing::info!("fetching all players.");
        let player_infos: Vec<PlayerInfo> = connection.get_player_infos().await?;
        state.set_players(player_infos);

        for player_info in state.get_players() {
            let now_playing = connection.get_now_playing_media(player_info.pid).await?;
            state.set_now_playing(player_info.pid, now_playing);

            let volume = connection.get_volume(player_info.pid).await?;
            state.set_player_volume(
                player_info.pid,
                Volume {
                    level: volume.level,
                    mute: On,
                },
            );

            let player_state = connection.get_play_state(player_info.pid).await?;
            state.set_player_state(player_state);
        }
        tracing::info!("fetched all players.");
        Ok(())
    }
}
impl Into<ApiCommand> for GetPlayers {
    fn into(self) -> ApiCommand {
        ApiCommand::GetPlayers(self)
    }
}
