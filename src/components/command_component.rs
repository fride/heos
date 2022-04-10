use bytes::Buf;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tracing::debug;
use tracing_subscriber::fmt::format;

use crate::{HeosError, HeosResult, PlayerUpdate};
use crate::connection::{CommandResponse, Connection};
use crate::model::{Level, PlayerId};
use crate::model::browse::MusicSource;
use crate::model::group::GroupInfo;
use crate::model::player::{PlayerInfo, PlayerNowPlayingMedia};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ApiCommand {
    GetPlayers,
    GetGroups,
    GetNowPlaying(PlayerId),
    GetMusicSources
}

pub async fn heos_api_component(mut connection: Connection,
                          mut command_receive:  Receiver<ApiCommand>,
                          results: Sender<PlayerUpdate>,
                          errors: Sender<HeosError>) -> HeosResult<()>{

    let _ = connection.write_frame("system/register_for_change_events?enable=off").await?;
    connection.read_command_response().await?;

    tokio::spawn(async move {
        while let Some(cmd) = command_receive.recv().await {
            let res = match cmd {
                ApiCommand::GetPlayers => get_players(&mut connection, &results).await,
                ApiCommand::GetGroups => get_groups(&mut connection, &results).await,
                ApiCommand::GetNowPlaying(pid) => get_now_playing(pid, &mut connection, &results).await,
                ApiCommand::GetMusicSources => {get_music_sources(&mut connection, &results).await}
            };
            if let Err(e) = res {
                let _ = errors.send(e).await;
            }
        }
    });
    Ok(())
}

async fn get_players(connection: &mut Connection, result_sender: &Sender<PlayerUpdate>) -> HeosResult<()> {
    let response: Vec<PlayerInfo> = send_command(connection, "player/get_players").await?;
    result_sender.send(PlayerUpdate::Players(response)).await;
    Ok(())
}

async fn get_groups(connection: &mut Connection, result_sender: &Sender<PlayerUpdate>) -> HeosResult<()> {
    let response: Vec<GroupInfo> = send_command(connection, "group/get_groups").await?;
    result_sender.send(PlayerUpdate::Groups(response)).await;
    Ok(())
}

async fn get_now_playing(player_id: PlayerId, connection: &mut Connection, result_sender: &Sender<PlayerUpdate>) -> HeosResult<()> {
    let response: PlayerNowPlayingMedia = send_command(connection, &format!("player/get_now_playing_media?pid={}", player_id)).await?;
    result_sender.send(PlayerUpdate::NowPlaying(response)).await;
    Ok(())
}

async fn get_music_sources(connection: &mut Connection, result_sender: &Sender<PlayerUpdate>) -> HeosResult<()> {
    let response: Vec<MusicSource> = send_command(connection, &format!("browse/get_music_sources")).await?;
    result_sender.send(PlayerUpdate::MusicSources(response)).await;
    Ok(())
}

async fn send_command<T>(connection: &mut Connection, command: &str) -> Result<T, HeosError>
    where
        T: TryFrom<CommandResponse, Error=HeosError>,
{
    let _ = connection.write_frame(command).await?;
    let res = connection.read_command_response().await?;
    res.try_into()
}

