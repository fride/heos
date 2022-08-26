use async_trait::async_trait;

use crate::command::{Command, CommandBehaviour, CommandCallback};
use crate::{Connection, HeosResult};
use crate::api::HeosApi;
use crate::model::player::{PlayerInfo, PlayerVolume, PlayState};
use crate::model::{Level, PlayerId};

#[derive(Debug, Clone)]
pub struct GetPlayerVolume {
    pub player_id: PlayerId
}
impl GetPlayerVolume {
    pub fn new(player_id: PlayerId) -> Self {
        Self {
            player_id
        }
    }
}

#[async_trait]
impl CommandBehaviour for GetPlayerVolume {
    type CommandResultType = PlayerVolume;

    async fn apply(self, connection: &mut Connection) -> HeosResult<Self::CommandResultType> {
        connection.get_volume(self.).await
    }

    fn to_command(self, callback: CommandCallback<Self::CommandResultType>) -> Command {
        Command::GetPlayerVolume(self, callback)
    }
}
