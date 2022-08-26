use async_trait::async_trait;

use crate::api::command::{Command, CommandBehaviour, CommandCallback};
use crate::{Connection, HeosResult};
use crate::api::HeosApi;
use crate::connection::CommandExecutor;
use crate::model::player::{PlayerInfo, PlayState};
use crate::model::{Level, PlayerId};

#[derive(Debug, Clone)]
pub struct GetPlayers;
impl GetPlayers {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CommandBehaviour for GetPlayers {
    type CommandResultType = Vec<PlayerInfo>;

    async fn apply(self, connection: &mut Connection) -> HeosResult<Self::CommandResultType> {
        connection.execute_command("player/get_players").await
    }

    fn to_command(self, callback: CommandCallback<Self::CommandResultType>) -> Command {
        Command::GetPlayers(self, callback)
    }
}
