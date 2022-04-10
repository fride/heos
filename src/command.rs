use std::fmt::format;

use tracing::debug;

use crate::{HeosError, HeosResult};
use crate::connection::{CommandResponse, Connection, EventResponse, Frame};
use crate::model::group::GroupInfo;
use crate::model::player::{PlayerInfo, PlayerVolume};
use crate::model::PlayerId;

pub trait HeosCommand {
    type CommandResult;
    fn get_payload(&self) -> String;
}

pub struct GetPlayers;
impl HeosCommand for GetPlayers {
    type CommandResult = Vec<PlayerInfo>;

    fn get_payload(&self) -> String {
        format!("{}/{}", "players", "get_players")
    }
}

pub struct GetGroups;
impl HeosCommand for GetGroups {
    type CommandResult = Vec<GroupInfo>;

    fn get_payload(&self) -> String {
        format!("{}/{}", "groups", "get_groups")
    }
}

pub struct GetPlayerVolume{pub pid: PlayerId}
impl HeosCommand for GetPlayerVolume {
    type CommandResult =PlayerVolume;
    fn get_payload(&self) -> String {
        format!("{}/{}?pid={}", "player", "get_volume", self.pid)
    }
}


// impl
enum EventOrResponse{
    Event(EventResponse),
    Command(CommandResponse)
}

pub struct CommandExecuter {
    connection: Connection
}

impl CommandExecuter {
    pub fn new(connection: Connection) -> Self {
        Self{connection}
    }

    pub async fn execute<T : HeosCommand>(&mut self, command: T) -> HeosResult<T::CommandResult>
        where
            T::CommandResult : TryFrom<CommandResponse, Error = HeosError>,{
        let _ = self.connection.write_frame(&command.get_payload()).await?;
        loop {
            match self.next_response().await? {
                EventOrResponse::Command(command) => {
                    let res: T::CommandResult = command.try_into()?;
                    return Ok(res);
                },
                EventOrResponse::Event(event) => {
                    debug!(">> got event '{}' while waiting for command", &event.event_name);
                }
            }
        }
    }

    async fn next_response(&mut self) -> Result<EventOrResponse, HeosError>
    {
        loop {
            let response = self.connection.read_frame().await?;
            match response {
                Some(Frame::Response(command)) => return Ok(EventOrResponse::Command(command)),
                Some(Frame::Error(error)) => return Err(HeosError::InvalidCommand{command : "".to_owned(), message : error}),
                Some(Frame::Event(event)) => return  Ok(EventOrResponse::Event(event)),
                Some(Frame::UnderProcess(command)) => {
                    debug!(">> waiting for {} to finish.", &command);
                },
                _ => {// nop
                }
            }
        }
    }
}
