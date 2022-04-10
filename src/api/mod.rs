use tokio::net::ToSocketAddrs;
use tokio::sync::mpsc;

pub use command_component::{ApiCommand, heos_api_component};
pub use event_componment::heos_event_component;

use self::connection::Connection;
use crate::model::browse::MusicSource;
use crate::model::group::GroupInfo;
use crate::model::player::{PlayerInfo, PlayerNowPlayingMedia};
use crate::model::{Level, Milliseconds, OnOrOff, PlayerId};
use crate::{HeosError, HeosResult};

mod command_component;
mod event_aggregator_component;
mod event_componment;

mod state_component;
pub(crate) mod parsers;
mod connection;

pub type HeosComponent = (
    mpsc::Sender<ApiCommand>,
    mpsc::Receiver<PlayerUpdate>,
    mpsc::Receiver<HeosError>,
);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PlayerUpdate {
    Players(Vec<PlayerInfo>),
    Groups(Vec<GroupInfo>),
    NowPlaying(PlayerNowPlayingMedia),
    PlayerVolume(PlayerId, Level),
    PlayingProgress(PlayerId, Milliseconds, Option<Milliseconds>),
    PlayerPlaybackError(PlayerId, String),
    PlayerVolumeChanged(PlayerId, Level, OnOrOff),
    MusicSources(Vec<MusicSource>),
}

pub async fn find() -> HeosResult<HeosComponent> {
    let mut connection = connection::Connection::find().await?;
    do_connect(connection).await
}

pub async fn connect<T: ToSocketAddrs>(c: T) -> HeosResult<HeosComponent> {
    let mut connection = connection::Connection::connect(c).await?;
    do_connect(connection).await

}
async fn do_connect(mut connection : connection::Connection) -> HeosResult<HeosComponent>  {
    let commend_connection = connection.try_clone().await?;
    println!("connected");
    // api calls
    let (command_send, command_receive) = mpsc::channel(32);
    // api results and event results
    let (response_send, response_receive) = mpsc::channel(32);
    let (event_send, event_receive) = mpsc::channel(32);
    let (error_send, error_receive) = mpsc::channel(32);

    let _ =
        event_componment::heos_event_component(connection, event_send, error_send.clone()).await?;
    command_component::heos_api_component(
        commend_connection,
        command_receive,
        response_send.clone(),
        error_send.clone(),
    )
        .await?;
    event_aggregator_component::heos_event_aggregator_component(
        event_receive,
        command_send.clone(),
        response_send,
        error_send,
    );

    Ok((command_send, response_receive, error_receive))
}
