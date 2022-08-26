use async_trait::async_trait;

use tokio::sync::oneshot;
use crate::{Connection, HeosResult};
use crate::api::HeosApi;
use crate::model::player::PlayerInfo;

pub type CommandCallback<T> = oneshot::Sender<HeosResult<T>>;
pub type CommandChannel = tokio::sync::mpsc::Sender<Command>;

pub struct CommandResult<R>(HeosResult<R>, CommandCallback<R>);

impl<R> CommandResult<R> {
    pub fn send_notification(self) -> bool {
        self.1.send(self.0)
            .is_ok()
    }
}

mod get_players;
pub use get_players::*;


// use structs to implement the logic of the individual cases and group them
// in this enum to send them via a channel.
pub enum Command{
    GetPlayers(GetPlayers)
}

impl Command {

    pub async fn apply(self, connection: &mut Connection) {
        match self {
            Command::GetPlayers(command) => {
                command.apply(connection).await.send_notification();
            }
        }
    }
}

pub async fn foo(command_channel: & CommandChannel) -> HeosResult<Vec<PlayerInfo>> {
    GetPlayers::schedule(command_channel).await
}
