use tokio::sync::oneshot;
use tokio::sync::mpsc;
use crate::{Connection, HeosError, HeosResult};
use crate::connection::CommandExecutor;
use crate::model::event::HeosEvent;
use crate::model::player::PlayerInfo;
use crate::model::PlayerId;
use async_trait::async_trait;
use tokio::sync::mpsc::Receiver;
use tokio_stream::Stream;
use tracing::event;
use crate::model::system::RegisteredForChangeEvents;

pub type Responder<T> = oneshot::Sender<T>;
pub type CommandChannel = mpsc::Sender<HeosCommand>;
pub type EventChannel = Receiver<HeosResult<HeosEvent>>;



// 
pub enum SystemCommand{
    RegisterForChangeEvents(Responder<mpsc::Receiver<HeosResult<HeosEvent>>>),
    UnRegisterForChangeEvents(Responder<mpsc::Receiver<HeosResult<()>>>),
}
pub enum PlayerCommand {
    GetPlayer(PlayerId,Responder<HeosResult<Vec<PlayerInfo>>>),
    GetPlayers(Responder<HeosResult<Vec<PlayerInfo>>>),
}

pub enum HeosCommand {
    GetPlayer(PlayerId,Responder<HeosResult<Vec<PlayerInfo>>>),
    GetPlayers(Responder<HeosResult<Vec<PlayerInfo>>>),
    RegisterForChangeEvents(Responder<mpsc::Receiver<HeosResult<HeosEvent>>>)
}

pub fn create_command_channel(connection: Connection) -> CommandChannel {
    let (a,mut commands) = mpsc::channel(13);
    let _ = tokio::spawn(async move {
        let mut ex = CommandExecuter {connection};
        while let Some(command) = commands.recv().await {
            let _ = ex.execute_command(command).await;
        }
    });
    a
}

struct CommandExecuter{
    pub connection: Connection
}

impl CommandExecuter {
    pub async fn execute_command(&mut self, command: HeosCommand) {
        match command {
            HeosCommand::GetPlayer(player_id, responder) => {
                let response = self.connection.execute_command(format!("player/get_player?pid={}", player_id)).await;
                let _ = responder.send(response);
            }
            HeosCommand::GetPlayers(responder) => {
                let response = self.connection.execute_command("player/get_players").await;
                let _ = responder.send(response);
            }

            // TODO this won't work
            // 1st clone connection
            // 2nd send '"system/register_for_change_events?enable=on"'
            // ???
            HeosCommand::RegisterForChangeEvents(responder) => {

                let (a,b) = mpsc::channel(4 );
                let _ = responder.send(b);
                let events = self.connection.try_clone()
                    .await
                    .map(|c| c.into_event_stream());
                match events {
                    Ok(e) => {
                        //todo this needs its own logik
                    }
                    Err(err) => {
                        a.send(Err(err)).await;
                    }
                }
            }
        }
    }
}
// todo make this lazy!
async fn create_event_channel(mut connection: Connection) -> HeosResult<EventChannel> {
    let (a, b) = mpsc::channel(13);
    let _ : RegisteredForChangeEvents = connection.execute_command("system/register_for_change_events?enable=on").await?;
    let _ = tokio::spawn(async move {
        loop {
            let event  = connection.read_event().await
                .and_then(|e| e.try_into());
            if let Err(_) = a.send(event).await {
                break;
            }
        }
    });
    Ok(b)
}

struct EventEmitter{
    pub connection: Connection,
    pub listeners: Vec<mpsc::Receiver<HeosResult<HeosEvent>>>
}

impl EventEmitter {

}
