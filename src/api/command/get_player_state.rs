use async_trait::async_trait;

use crate::api::command::{Command, CommandBehaviour, CommandCallback};
use crate::{Connection, HeosResult};
use crate::connection::CommandExecutor;
use crate::model::player::{PlayerInfo, PlayerPlayState, PlayState};
use crate::model::{Level, PlayerId};

#[derive(Debug, Clone)]
pub struct GetPlayerState {player_id: PlayerId}
impl GetPlayerState {
    pub fn new(player_id: PlayerId) -> Self {
        Self { player_id}
    }
}

#[async_trait]
impl CommandBehaviour for GetPlayerState {
    type CommandResultType = PlayerPlayState;

    async fn apply(self, connection: &mut Connection) -> HeosResult<Self::CommandResultType> {
        connection.execute_command(format!("player/get_play_state?pid={pid}", pid = self.player_id))
            .await
    }

    fn to_command(self, callback: CommandCallback<Self::CommandResultType>) -> Command {
        Command::GetPlayerState(self, callback)
    }
}
