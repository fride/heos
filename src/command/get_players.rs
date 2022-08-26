use async_trait::async_trait;

use crate::command::{Command, CommandBehaviour, CommandCallback};
use crate::{Connection, HeosResult};
use crate::api::HeosApi;
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
        connection.get_player_infos().await
    }

    fn to_command(self, callback: CommandCallback<Self::CommandResultType>) -> Command {
        Command::GetPlayers(self, callback)
    }
}
