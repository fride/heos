use async_stream::stream;
use async_stream::try_stream;
use tokio_stream::Stream;
use tracing::{debug, instrument};

use crate::HeosError;
use crate::model::browse::*;
use crate::model::event::*;
use crate::model::player::*;
use crate::model::system::*;

use super::connection::*;
use super::parsers::*;

pub struct Api {
    connection: Connection,
}

enum EventOrResponse{
    Event(EventResponse),
    Command(CommandResponse)
}

impl Api {
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }

    pub async fn register_for_change_events(
        &mut self,
    ) -> Result<RegisteredForChangeEvents, HeosError> {
        let res = self
            .send_command("system/register_for_change_events?enable=on")
            .await?;
        Ok(res)
    }

    pub async fn account_check(&mut self) -> Result<AccountState, HeosError> {
        let res = self.send_command("system/check_account").await?;
        Ok(res)
    }

    pub async fn get_players(&mut self) -> Result<Vec<PlayerInfo>, HeosError> {
        let players: Vec<PlayerInfo> = self.send_command("player/get_players").await?;
        Ok(players)
    }

    pub async fn get_music_sources(&mut self) -> Result<Vec<MusicSource>, HeosError> {
        let res: Vec<MusicSource> = self.send_command("browse/get_music_sources").await?;
        Ok(res)
    }

    /// Convert the subscriber into a `Stream` yielding new messages published
    /// on subscribed channels.
    ///
    /// `Subscriber` does not implement stream itself as doing so with safe code
    /// is non trivial. The usage of async/await would require a manual Stream
    /// implementation to use `unsafe` code. Instead, a conversion function is
    /// provided and the returned stream is implemented with the help of the
    /// `async-stream` crate.
    pub fn into_stream(mut self) -> impl Stream<Item = crate::HeosResult<HeosEvent>> {
        // Uses the `try_stream` macro from the `async-stream` crate. Generators
        // are not stable in Rust. The crate uses a macro to simulate generators
        // on top of async/await. There are limitations, so read the
        // documentation there.
        try_stream! {
            let _ = self.register_for_change_events().await?;
            loop {
                let event : HeosEvent = self.next_event().await?;
                yield(event);
            }
        }
    }

    async fn send_command<T>(&mut self, command: &str) -> Result<T, HeosError>
        where
            T: TryFrom<CommandResponse, Error = HeosError>,
    {
        let _ = self.connection.write_frame(command).await?;
        loop {
            match self.next_response().await? {
                EventOrResponse::Command(command) => {
                    let res: T = command.try_into()?;
                    return Ok(res);
                },
                EventOrResponse::Event(event) => {
                    debug!(">> got event '{}' while waiting for command", &event.event_name);
                }
            }
        }
    }
    async fn next_event(&mut self) -> Result<HeosEvent, HeosError> {
        loop {
            if let EventOrResponse::Event(event) =  self.next_response().await? {
                return event.try_into();
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
