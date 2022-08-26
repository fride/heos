use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};
use crate::{Connection, HeosResult};
use crate::api::HeosApi;
use crate::model::player::{PlayerInfo, PlayerPlayState, PlayerVolume};
use crate::model::group::GroupInfo;

// the fine commands:
mod get_players;
pub use get_players::*;

mod get_player_volume;
pub use get_player_volume::*;

mod get_player_state;
pub use get_player_state::*;


pub type CommandCallback<T> = oneshot::Sender<HeosResult<T>>;

#[async_trait]
pub trait CommandBehaviour {
    type CommandResultType;

    async fn apply(self, connection: &mut Connection) -> HeosResult<Self::CommandResultType>;

    fn to_command(self, callback: CommandCallback<Self::CommandResultType>) -> Command;
}

// Main component here!
// TODO this is basically a HeosApi that .....
pub struct CommandChannel(mpsc::Sender<Command>);

impl CommandChannel {
    pub fn new(mut connection: Connection) -> Self {
        tracing::info!("Setting up command_handler");
        let (command_channel, mut api_receiver) = mpsc::channel::<Command>(16);

        let _join = tokio::spawn(async move {
            tracing::info!("waiting for commands.");
            while let Some(command) = api_receiver.recv().await {
                tracing::info!("received command");
                let _ = command.apply(&mut connection).await;

            }
            tracing::info!("command listener done.");
        });
        Self(command_channel)
    }

    pub async fn schedule<C, R>(&self, command: C) -> HeosResult<R>
        where  C : CommandBehaviour<CommandResultType = R> + Sized + Send,
               R : Send + Sized{
        let (s,r) = oneshot::channel();
        let command = command.to_command(s);
        let _ = self.0.send(command).await.expect("Failed to send command"); // TODO failure!?
        r.await.unwrap()
    }
}
impl Into<CommandChannel> for Connection {
    fn into(self) -> CommandChannel {
        CommandChannel::new(self)
    }
}

#[derive(Debug)]
pub enum Command {
    GetPlayers(GetPlayers, CommandCallback<Vec<PlayerInfo>>),
    GetPlayerVolume(GetPlayerVolume, CommandCallback<PlayerVolume>),
    GetPlayerState(GetPlayerState, CommandCallback<PlayerPlayState>)
    //GetGroups(GetGroups, CommandCallback<Vec<GroupInfo>>)
}

impl Command {
    pub async fn apply(self,connection: &mut Connection) {
        let res = match self {
            Command::GetPlayers(command, callback) =>
                callback.send(command.apply(connection).await).is_ok(),
            Command::GetPlayerVolume(command, callback) =>
                callback.send(command.apply(connection).await).is_ok(),
            Command::GetPlayerState(command, callback) =>
                callback.send(command.apply(connection).await).is_ok(),
        };
    }
}
