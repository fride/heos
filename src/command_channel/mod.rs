use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::HeosResult;
use crate::model::player::PlayerInfo;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum HeosCommand {
    GetPlayers,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum HeosResponse {
    Players(Vec<PlayerInfo>)
}
type CommandChannel = (Sender<HeosCommand>, Receiver<HeosResult<HeosResponse>>);

pub async fn command_channel() -> CommandChannel {
    let (command_send,mut command_rcv) = mpsc::channel(32);
    let (response_send,mut response_rcv) = mpsc::channel(32);

    let _ = tokio::spawn(async move {
        loop {
            let cmd = command_rcv.recv().await.unwrap();
            match cmd {
                HeosCommand::GetPlayers => {
                    let _ =  response_send.send(Ok(HeosResponse::Players(Vec::default()))).await.expect("Could not send response");
                }
            }
        }
    });
    (command_send, response_rcv)
}
