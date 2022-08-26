use im::Vector;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::api::HeosApi;
use crate::driver::{ApiCommand, StateUpdates};
use crate::model::group::GroupInfo;
use crate::model::player::PlayerInfo;
use crate::model::{GroupId, PlayerId};
use crate::{Connection, HeosError};

pub fn create_command_handler(
    mut connection: Connection,
    mut commands: Receiver<ApiCommand>,
    results: Sender<StateUpdates>,
) {
    tracing::info!("Setting up create_command_handler");
    tokio::spawn(async move {
        while let Some(command) = commands.recv().await {
            handle_command(command, &mut connection, &results).await;
        }
        tracing::info!("Finished command thread.");
    });
}

async fn handle_command(
    command: ApiCommand,
    connection: &mut Connection,
    results: &Sender<StateUpdates>,
) {
    tracing::debug!("Received command: {:?}", &command);
    let command_str = format!("{:?}", &command);
    let response = match command {
        ApiCommand::GetPlayers => {
            let response = connection.get_player_infos().await;
            response.map(|res| vec![StateUpdates::Players(res)])
        }
        ApiCommand::GetGroups => {
            let response = connection.get_groups().await;
            response.map(|res| vec![StateUpdates::Groups(res)])
        }
        ApiCommand::RefreshState => load_state(connection).await,
        ApiCommand::LoadPlayerVolume(pid) => connection
            .get_volume(pid)
            .await
            .map(|v| vec![StateUpdates::PlayerVolumes(v)]),
        ApiCommand::LoadGroupVolume(gid) => {
            let volume = connection.get_group_volume(gid).await;
            volume.map(|v| vec![StateUpdates::GroupVolumes(v)])
        }
        ApiCommand::LoadNowPLaying(pid) => connection
            .get_now_playing_media(pid)
            .await
            .map(|now| vec![StateUpdates::PlayerNowPlaying(pid.clone(), now)]),
    };
    match response {
        Ok(responses) => {
            for response in responses {
                results
                    .send(response)
                    .await
                    .expect("Failed to send api response");
            }
        }
        Err(err) => {
            println!("Command {} failed! {:?}", command_str, &err);
            results
                .send(StateUpdates::Error(err))
                .await
                .expect("failed to send error response");
        }
    }
}

async fn load_state(connection: &mut Connection) -> Result<Vec<StateUpdates>, HeosError> {
    tracing::debug!("Loading state");
    let mut responses = vec![];
    let players: Vec<PlayerInfo> = connection.get_player_infos().await?;
    let groups: Vec<GroupInfo> = connection.get_groups().await?;
    let pids: Vector<PlayerId> = players.iter().map(|p| p.pid).collect();
    let gids: Vector<GroupId> = groups.iter().map(|p| p.gid).collect();
    tracing::debug!("Found {} players and {} groups", pids.len(), gids.len());
    responses.push(StateUpdates::Players(players));
    responses.push(StateUpdates::Groups(groups));

    for pid in &pids {
        let now_playing = connection.get_now_playing_media(pid.clone()).await?;
        responses.push(StateUpdates::PlayerNowPlaying(pid.clone(), now_playing));
        let player_volume = connection.get_volume(pid.clone()).await?;
        responses.push(StateUpdates::PlayerVolumes(player_volume));
    }
    for gid in &gids {
        let group_volume = connection.get_group_volume(gid.clone()).await?;
        responses.push(StateUpdates::GroupVolumes(group_volume));
    }
    tracing::debug!("Loading state succeeded.");
    Ok(responses)
}
