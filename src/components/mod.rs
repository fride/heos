use tokio::sync::mpsc;

pub use command_component::{ApiCommand, heos_api_component};
pub use event_componment::heos_event_component;

use crate::{HeosError, HeosEvent, HeosResult};
use crate::connection::Connection;
use crate::model::{Level, Milliseconds, OnOrOff, PlayerId};
use crate::model::browse::MusicSource;
use crate::model::group::GroupInfo;
use crate::model::player::{PlayerInfo, PlayerNowPlayingMedia};

mod event_componment;
mod command_component;
mod event_aggregator_component;


mod state_component;

pub type HeosComponent = (mpsc::Sender<ApiCommand>, mpsc::Receiver<PlayerUpdate>, mpsc::Receiver<HeosError>);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PlayerUpdate {
    Players(Vec<PlayerInfo>),
    Groups(Vec<GroupInfo>),
    NowPlaying(PlayerNowPlayingMedia),
    PlayerVolume(PlayerId, Level),
    PlayingProgress(PlayerId,Milliseconds,Option<Milliseconds>),
    PlayerPlaybackError(PlayerId, String),
    PlayerVolumeChanged(PlayerId, Level,OnOrOff),
    MusicSources(Vec<MusicSource>)
}

pub async fn heos_components(mut connection: Connection) -> HeosResult<HeosComponent>{
    let commend_connection = connection.try_clone().await?;
    // api calls
    let (command_send, command_receive) = mpsc::channel(32);
    // api results and event results
    let (response_send, response_receive) = mpsc::channel(32);
    let (event_send, event_receive) = mpsc::channel(32);
    let (error_send, error_receive) = mpsc::channel(32);

    let _ = event_componment::heos_event_component(connection, event_send, error_send.clone()).await?;
    command_component::heos_api_component(commend_connection, command_receive, response_send.clone(), error_send.clone()).await?;
    event_aggregator_component::heos_event_aggregator_component(event_receive, command_send.clone(), response_send, error_send);

    Ok((command_send, response_receive, error_receive))
}

//
//
//
// /////// grober unfug folgt!
//
// pub type ApiResponseChannel = Sender<Sender<ApiResponse>>;
//
// // this makes the mpsc behave like a pub sub channel. I don't know if this is a good idea. ;)
// pub fn api_response_channel_foo(mut source: Receiver<ApiResponse>) -> ApiResponseChannel {
//     let (tx_senders, mut rx_senders) = mpsc::channel(32);
//
//     tokio::spawn(async move {
//         let mut senders: Vec<Sender<ApiResponse>> = Vec::default();
//         tokio::select! {
//             Some(new_sender) = rx_senders.recv() => { senders.push(new_sender) }
//             Some(response) = source.recv() => {
//                 for sender in senders.iter() {
//                     sender.send(response.clone()).await;
//                 }
//             }
//         }
//     });
//     tx_senders
//
// }

// macros sind cool! https://blog.logrocket.com/macros-in-rust-a-tutorial-with-examples/
// type ErrorOutPort = Receiver<HeosError>;
// type ErrorInPort = Sender<HeosError>;
//
// pub struct Port<T> {
//     output: Sender<T>,
//     input: Receiver<T>,
// }
//
// fn make_multicast() -> (Sender<String>, Sender<Sender<String>>) {
//     let (tx1, mut rx1) = mpsc::channel::<String>(32);
//     let (tx2, mut rx2) = mpsc::channel(4);
//
//
//     tokio::spawn(async move {
//         let mut senders: Vec<Sender<String>> = Vec::default();
//         tokio::select! {
//             Some(value_to_send) = rx1.recv() => {
//                 for sender in senders {
//                     let  _ = sender.send(value_to_send.clone()).await;
//                 }
//             }
//             Some(new_out_channel) = rx2.recv() => {
//                 senders.push(new_out_channel);
//             }
//         }
//     });
//     (tx1, tx2)
// }
//
//
// pub async fn multicast<A>(mut channel: Receiver<A>, senders: Vec<Sender<A>>) where A: Send + Clone + 'static {
//     tokio::spawn(async move {
//         while let Some(a) = channel.recv().await {
//             for sender in senders.iter() {
//                 let copy = a.clone();
//                 let _ = sender.send(copy).await;
//             }
//         }
//     });
// }
//
// pub fn map_channel<A, B>(mut channel: Receiver<A>, handler: fn(A) -> B) -> Receiver<B>
//     where A: Send + 'static,
//           B: Send + 'static { // the compiler tells me to be static. but why!?
//     let (tx1, mut rx1) = mpsc::channel::<B>(32);
//     tokio::spawn(async move {
//         while let Some(a) = channel.recv().await {
//             let res = handler(a);
//             tx1.send(res).await;
//         }
//     });
//     rx1
// }
//
// pub async fn do_silly_things() {
//     let (tx1, mut rx1) = mpsc::channel(32);
//     let mut plus_one = map_channel(rx1, |a: i64| a + 1);
//     tokio::spawn(async move {
//         while let Some(p) = plus_one.recv().await {
//             println!("Got : {}", p);
//         }
//     });
//     tokio::spawn(async move {
//         println!("Send 2");
//         tx1.send(2).await;
//         println!("Send 23");
//         tx1.send(23).await;
//         println!("Send 24");
//         tx1.send(24).await;
//     });
// }
