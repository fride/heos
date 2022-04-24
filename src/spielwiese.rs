use async_trait::async_trait;
use std::fmt::Display;
use std::marker::PhantomData;

use crate::connection::{CommandResponse, Connection};

use crate::model::group::GroupInfo;
use crate::model::player::{PlayState, PlayerInfo, PlayerPlayState};
use crate::model::PlayerId;
use crate::{HeosError, HeosResult};

// inspired by https://doc.rust-lang.org/nomicon/hrtb.html
// and https://doc.rust-lang.org/std/marker/struct.PhantomData.html
// https://betterprogramming.pub/rust-events-revisited-926486721e3f
pub struct HeosCommand<R> {
    payload: String,
    phantom: PhantomData<R>,
}

pub struct HeosAction<R, A> {
    payload: String,
    action: A,
    phantom: PhantomData<R>,
}

impl<R, A> HeosAction<R, A>
where
    A: FnMut(R) -> (),
    A: Send + 'static,
    R: TryFrom<CommandResponse, Error = HeosError> + Send + std::marker::Sync,
{
    pub fn new<P: Display>(payload: P, action: A) -> Self {
        Self {
            payload: format!("{}", payload),
            phantom: PhantomData,
            action,
        }
    }

    pub async fn execute(&mut self, connection: &mut Connection) -> HeosResult<()> {
        let _ = connection.write_frame(&self.payload).await?;
        let response = connection.read_command_response().await?;
        let real_response: R = response.try_into()?;
        (self.action)(real_response);
        Ok(())
    }
}

#[async_trait]
pub trait ExecutableHeosCommand {
    type ResultType;
    async fn execute(&self, connection: &mut Connection) -> HeosResult<Self::ResultType>;
}

#[async_trait]
impl<R> ExecutableHeosCommand for HeosCommand<R>
where
    R: TryFrom<CommandResponse, Error = HeosError> + Send + std::marker::Sync,
{
    type ResultType = R;

    async fn execute(&self, connection: &mut Connection) -> HeosResult<Self::ResultType> {
        let _ = connection.write_frame(&self.payload).await?;
        let response = connection.read_command_response().await?;
        response.try_into()
    }
}

impl<R> HeosCommand<R> {
    pub fn new<A: Display>(payload: A) -> HeosCommand<R> {
        HeosCommand {
            payload: format!("{}", payload),
            phantom: PhantomData,
        }
    }
}

pub fn get_players() -> HeosCommand<Vec<PlayerInfo>> {
    HeosCommand {
        payload: "player/get_players".to_owned(),
        phantom: PhantomData,
    }
}
pub fn get_player_state(pid: &PlayerId) -> HeosCommand<PlayerPlayState> {
    HeosCommand {
        payload: format!("player/get_play_state?pid={pid}", pid = pid),
        phantom: PhantomData,
    }
}
pub fn set_player_state(pid: &PlayerId, state: PlayState) -> HeosCommand<Vec<PlayerInfo>> {
    HeosCommand {
        payload: format!(
            "player/set_play_state?pid={pid}&state={state}",
            pid = pid,
            state = state
        ),
        phantom: PhantomData,
    }
}

pub fn get_groups() -> HeosCommand<Vec<GroupInfo>> {
    HeosCommand {
        payload: "group/get_groups".to_owned(),
        phantom: PhantomData,
    }
}

//
// #[async_trait]
// pub trait HeosCommand {
//     type CommandResult;
//
//     fn command_payload(&self) -> String;
//
//     async fn execute(&self, connection: &mut Connection) -> HeosResult<CommandResult>
//     where  Self::CommandResult: TryFrom<CommandResponse, Error = HeosError>{
//         execute_command(self, connection).await
//     }
// }
//
// async fn execute_command<C,R>(command: C, connection: &mut Connection) -> HeosResult<R>
//     where C : HeosCommand< CommandResult = R>,
//     R : TryFrom<CommandResponse, Error = HeosError>
// {
//     let cmd = format!("heos//{}\r\n", command.command_payload());
//     let _ = connection.write_frame(&cmd).await?;
//     let response = connection.read_command_response().await?;
//     response.try_into()
// }
