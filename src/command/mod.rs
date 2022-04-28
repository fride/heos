use std::fmt::Display;
use std::marker::PhantomData;

use async_trait::async_trait;
use tokio::sync::oneshot;

use crate::connection::CommandResponse;
use crate::model::player::{PlayerInfo, PlayerVolume};
use crate::model::PlayerId;
use crate::{Connection, HeosError, HeosResult};

#[derive(Debug)]
pub struct HeosCommand<R> {
    payload: String,
    return_type: PhantomData<R>,
}

impl<R> HeosCommand<R> {
    fn create<A: Display>(payload: A) -> Self {
        Self {
            payload: format!("{}", payload),
            return_type: PhantomData::default(),
        }
    }
}

pub fn get_players_command() -> HeosCommand<Vec<PlayerInfo>> {
    HeosCommand::create("player/get_players")
}

pub fn get_player_volume(player_id: PlayerId) -> HeosCommand<PlayerVolume> {
    HeosCommand::create(format!("player/get_volume?pid={}", player_id))
}

#[async_trait]
pub trait ExecutableHeosCommand {
    type CommandResultType;
    async fn parse_payload(
        &self,
        connection: &mut Connection,
    ) -> HeosResult<Self::CommandResultType>;
}

#[async_trait]
impl<R> ExecutableHeosCommand for HeosCommand<R>
where
    R: TryFrom<CommandResponse, Error = HeosError> + Send + std::marker::Sync,
{
    type CommandResultType = R;

    async fn parse_payload(
        &self,
        connection: &mut Connection,
    ) -> HeosResult<Self::CommandResultType> {
        let _ = connection.write_frame(&self.payload).await?;
        let response = connection.read_command_response().await?;
        let r: R = response.try_into()?;
        Ok(r)
    }
}

pub async fn handle_command<R>(
    connection: &mut Connection,
    command: HeosCommand<R>,
    handler: tokio::sync::oneshot::Sender<HeosResult<R>>,
) where
    R: TryFrom<CommandResponse, Error = HeosError> + Send + std::marker::Sync,
{
    let response = command.parse_payload(connection).await;
    let _ = handler.send(response);
}

pub mod different_structs {
    use std::fmt;
    use std::fmt::Display;
    use std::marker::PhantomData;

    use async_trait::async_trait;
    use tokio::sync::mpsc;

    use crate::connection::CommandResponse;
    use crate::model::group::{GroupInfo, GroupVolume};
    use crate::model::player::{PlayerInfo, PlayerVolume};
    use crate::model::{GroupId, PlayerId};
    use crate::{Connection, HeosError, HeosResult};

    pub trait HeosCommand: Display {
        type CommandResultType;
    }

    #[async_trait]
    pub trait HeosCommandExecutor {
        async fn execute<C, R>(&mut self, command: C) -> HeosResult<R>
        where
            C: HeosCommand<CommandResultType = R> + Send + std::marker::Sync,
            R: TryFrom<CommandResponse, Error = HeosError> + Send + std::marker::Sync;
    }

    #[async_trait]
    impl HeosCommandExecutor for Connection {
        async fn execute<C, R>(&mut self, command: C) -> HeosResult<R>
        where
            C: HeosCommand<CommandResultType = R> + Send + std::marker::Sync,
            R: TryFrom<CommandResponse, Error = HeosError> + Send + std::marker::Sync,
        {
            let _ = self.write_frame(&format!("{}", &command)).await?;
            let response = self.read_command_response().await?;
            let r: R = response.try_into()?;
            Ok(r)
        }
    }

    macro_rules! command {
        (
            $struct_name:ident :: $command:expr => $result_type:ty
        ) => {
            #[derive(Debug)]
            pub struct $struct_name;
            impl fmt::Display for $struct_name {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, "{}", $command)
                }
            }
            impl HeosCommand for $struct_name {
                type CommandResultType = $result_type;
            }

        };
        (
            $struct_name:ident {
            $(
                $field_vis:vis $field_name:ident : $field_type:ty
            ),*
            } :: $command:expr => $result_type:ty
        ) => {
            #[derive(Debug)]
            pub struct $struct_name{
                $(
                pub $field_name : $field_type,
                )*
            }
            impl fmt::Display for $struct_name {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    let mut params = vec![];
                    $(
                        params.push(format!("{}={}", stringify!($field_name), self.$field_name ));
                    ),*
                    write!(f, "{}?{}", $command, params.join("&"))
                }
            }
            impl HeosCommand for $struct_name {
                type CommandResultType = $result_type;
            }
        };
    }
    // todo maybe make this a bit nicer to read?
    command!( GetPlayers :: "player/get_players" => Vec<PlayerInfo>);
    command!( GetPlayerVolume {pid: PlayerId} ::"player/get_volume" => PlayerVolume);
    command!( GetGroups  ::"group/get_groups" => Vec<GroupInfo>);
    command!( GetGroupVolume  {gid: GroupId} ::"group/get_volume" => GroupVolume);

    #[derive(Debug)]
    pub enum HeosCommands {
        GetPlayers(GetPlayers),
        GetPlayerVolume(GetPlayerVolume),
        GetGroups(GetGroups),
        GetGroupVolume(GetGroupVolume),
    }

    #[derive(Debug)]
    pub enum HeosCommandResult {
        PlayerInfos(Vec<PlayerInfo>),
        PlayerVolume(PlayerVolume),
        Groups(Vec<GroupInfo>),
        GroupVolume(GroupVolume),
    }

    pub fn create_command_handler(
        mut connection: Connection,
    ) -> (
        mpsc::Sender<HeosCommands>,
        mpsc::Receiver<HeosCommandResult>,
        mpsc::Receiver<HeosError>,
    ) {
        let (command_send, mut r) = tokio::sync::mpsc::channel(12);
        let (result_send, result_receive) = tokio::sync::mpsc::channel(12);
        let (error_send, error_receive) = tokio::sync::mpsc::channel(12);
        tokio::spawn(async move {
            while let Some(cmd) = r.recv().await {
                //todo this could be a macro too! ;)
                let result = match cmd {
                    HeosCommands::GetPlayers(get_players) => connection
                        .execute(get_players)
                        .await
                        .map(|r| HeosCommandResult::PlayerInfos(r)),
                    HeosCommands::GetPlayerVolume(get_player_volume) => connection
                        .execute(get_player_volume)
                        .await
                        .map(|r| HeosCommandResult::PlayerVolume(r)),
                    HeosCommands::GetGroups(get_groups) => connection
                        .execute(get_groups)
                        .await
                        .map(|r| HeosCommandResult::Groups(r)),
                    HeosCommands::GetGroupVolume(get_group_volume) => connection
                        .execute(get_group_volume)
                        .await
                        .map(|r| HeosCommandResult::GroupVolume(r)),
                };
                let _ = match result {
                    Ok(res) => {
                        result_send.send(res).await;
                    }
                    Err(err) => {
                        error_send.send(err).await;
                    }
                };
            }
        });
        (command_send, result_receive, error_receive)
    }

    #[cfg(test)]
    mod tests {
        use actix_web::web::get;

        use super::*;

        #[test]
        pub fn test_generated_display() {
            let get_players = GetPlayers;
            assert_eq!("player/get_players", format!("{}", &get_players));
            let get_volume = GetPlayerVolume { pid: -12 };
            assert_eq!("player/get_volume?pid=-12", format!("{}", &get_volume));
        }
    }
}
