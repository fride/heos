use tokio::sync::{oneshot};
use tokio::sync::mpsc;


pub use get_music_sources::GetMusicSources;
pub use get_players::GetPlayers;
pub use init::InitController;

use crate::{Connection, HeosResult};

use crate::contoller::State;


// https://github.com/tokio-rs/mini-redis/blob/master/src/cmd/get.rs
// can I twist this to make my code nicer.

mod get_players;
// mod get_groups;
// pub use get_groups::GetGroups;

mod get_music_sources;
mod init;

pub type CommandNotifier = oneshot::Sender<HeosResult<()>>;

#[derive(Debug)]
pub enum ApiCommand {
    GetPlayers(GetPlayers),
    GetMusicSources(GetMusicSources),
    InitController(InitController),
}

impl ApiCommand {
    pub async fn apply(self, connection: &mut Connection, state: &State) -> HeosResult<()> {
        match self {
            ApiCommand::GetPlayers(get_players) => get_players.apply(connection, state).await,
            ApiCommand::GetMusicSources(get_music_sources) => {
                get_music_sources.apply(connection, state).await
            }
            ApiCommand::InitController(init) => init.apply(connection, state).await,
        }
    }
}

#[derive(Debug)]
pub struct CommandChannel(mpsc::Sender<(ApiCommand, Option<CommandNotifier>)>);

impl CommandChannel {
    pub fn new(mut connection: Connection, state: State) -> Self {
        tracing::info!("Setting up command_handler");
        let (command_channel, mut api_receiver) =
            mpsc::channel::<(ApiCommand, Option<CommandNotifier>)>(16);

        let _join = tokio::spawn(async move {
            tracing::info!("waiting for commands.");
            while let Some((command, notify)) = api_receiver.recv().await {
                tracing::info!("received command");
                let n = command.apply(&mut connection, &state).await;
                if let Some(notify) = notify {
                    tracing::info!("command listener sending ack.");
                    let _ = notify.send(n);
                }
            }
            tracing::info!("command listener done.");
        });

        Self(command_channel)
    }

    pub async fn send<A: Into<ApiCommand>>(&self, command: A) -> HeosResult<()> {
        tracing::debug!("Enqueue command. {:?}", self.0.capacity());
        let command = command.into();
        let _res = self.0.send((command, None)).await.unwrap();
        Ok(())
    }

    pub async fn send_ack<A: Into<ApiCommand>>(
        &self,
        command: A,
        command_ack: CommandNotifier,
    ) -> HeosResult<()> {
        tracing::debug!("Enqueue command. {:?}", self.0.capacity());
        let command = command.into();
        let _ = self.0.send((command, Some(command_ack))).await;
        Ok(())
    }

    pub fn clone(&self) -> Self {
        CommandChannel(self.0.clone())
    }
}

#[cfg(test)]
mod tests {
    

    #[tokio::test]
    pub async fn stuff() {
        // let (cmd, rec) = GetPlayers{};
        // let (s,r) = mpsc::channel(3);
        // let channel = CommandChannel(s);
        // channel.send(cmd).await.unwrap();
    }
}
