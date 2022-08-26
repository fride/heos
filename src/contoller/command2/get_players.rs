use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::{CommandChannel, Connection, HeosResult};
use crate::api::HeosApi;
use crate::contoller::command2::{Command, CommandCallback, CommandResult};
use crate::model::player::PlayerInfo;

pub struct GetPlayers {
    // this is much simpler then async callbacks :D
    callback: CommandCallback<Vec<PlayerInfo>>
}

impl GetPlayers {

    pub fn new() -> (Self, oneshot::Receiver<HeosResult<Vec<PlayerInfo>>>) {
        let (callback,r) = oneshot::channel();
        (Self {callback}, r)
    }
    pub async fn schedule(command_channel: & CommandChannel) -> HeosResult<Vec<PlayerInfo>> {
        let (command, callback) = GetPlayers::new();
        let _ = command_channel.send(command.into()).await; // TODO simply panik if this goes wrong?
        callback.await?
    }

    pub async fn apply(self, connection: &mut Connection) -> CommandResult<Vec<PlayerInfo>>{
        let res = connection.get_player_infos().await;
        CommandResult(res, self.callback)
    }
}

impl Into<Command> for GetPlayers {
    fn into(self) -> Command {
        Command::GetPlayers(self)
    }
}
