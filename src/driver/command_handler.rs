use im::Vector;
use tokio::sync::mpsc::{Sender, Receiver};
use crate::{Connection, HeosError};
use crate::driver::{ApiCommand, ApiResults, Shared};
use crate::driver::state::DriverState;
use crate::model::group::{GroupInfo, GroupVolume};
use crate::model::player::PlayerInfo;
use crate::model::{GroupId, PlayerId};
use crate::api::HeosApi;

pub fn create_command_handler(
    mut connection: Connection,
    mut commands: Receiver<ApiCommand>,
    results: Sender<ApiResults>,
) {
    println!("Setting up create_command_handler");
    tokio::spawn(async move {
        println!("Waiting for commands ");
        while let Some(command) = commands.recv().await {
            println!("Got command {:?}", &command);
            handle_command(command, &mut connection, &results).await;
        }
    });
}


async fn handle_command(
    command: ApiCommand,
    connection: &mut Connection,
    results: &Sender<ApiResults>,
) {
    let response = match command {
        ApiCommand::GetPlayers => {
            let response = connection.load_players().await;
            response.map(|res| vec![ApiResults::Players(res)])
        }
        ApiCommand::GetGroups => {
            let response = connection.get_groups().await;
            response.map(|res| vec![ApiResults::Groups(res)])
        }
        ApiCommand::RefreshState => load_state(connection).await,
        ApiCommand::LoadPlayerVolume(pid) => connection
            .get_volume(pid)
            .await
            .map(|v| vec![ApiResults::PlayerVolumes(v)]),
        ApiCommand::LoadGroupVolume(gid) => {
            let volume = connection.get_group_volume(gid).await;
            volume.map(|v| vec![ApiResults::GroupVolumes(v)])
        }
        ApiCommand::LoadNowPLaying(pid) => connection
            .get_now_playing_media(pid)
            .await
            .map(|now| vec![ApiResults::PlayerNowPlaying(now)]),
    };
    match response {
        Ok(responses) => {
            for response in responses {
                results.send(response).await;
            }
        }
        Err(err) => {
            println!("Command failed! {:?}", &err);
            results.send(ApiResults::Error(err)).await;
        }
    }
}

async fn load_state(connection: &mut Connection) -> Result<Vec<ApiResults>, HeosError> {
    let mut responses = vec![];
    let players: Vec<PlayerInfo> = connection.load_players().await?;
    let groups: Vec<GroupInfo> = connection.get_groups().await?;
    let pids: Vector<PlayerId> = players.iter().map(|p| p.pid).collect();
    let gids: Vector<GroupId> = groups.iter().map(|p| p.gid).collect();
    responses.push(ApiResults::Players(players));
    responses.push(ApiResults::Groups(groups));

    for pid in &pids {
        let now_playing = connection.get_now_playing_media(pid.clone()).await?;
        responses.push(ApiResults::PlayerNowPlaying(now_playing));
        let player_volume = connection.get_volume(pid.clone()).await?;
        responses.push(ApiResults::PlayerVolumes(player_volume));
    }
    for gid in &gids {
        let group_volume = connection.get_group_volume(gid.clone()).await?;
        responses.push(ApiResults::GroupVolumes(group_volume));
    }
    Ok(responses)
}
