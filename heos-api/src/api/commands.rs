use crate::api::CommandChannel;
use crate::{HeosError, HeosResult};
use crate::types::group::Group;
use crate::types::player::{HeosPlayer, PlayerInfo};

pub async fn load_groups(channel: &CommandChannel) -> HeosResult<Vec<Group>> {
    let mut groups = vec![];
    let group_infos = channel.get_groups().await?;
    for group_info in group_infos {
        let volume = channel.get_group_volume(group_info.gid).await?;
        groups.push(Group {
            name: group_info.name,
            gid: group_info.gid,
            volume: volume.level,
            players: group_info.players,
        });
    }
    Ok(groups)
}

pub async fn load_players(channel: &CommandChannel) -> Result<Vec<HeosPlayer>, HeosError> {
    let mut players = vec![];
    let player_infos = channel.get_player_infos().await?;
    for info in player_infos {
        players.push(fetch_player(channel, info).await?)
    }
    Ok(players)
}

async fn fetch_player(channel: &CommandChannel, info: PlayerInfo) -> HeosResult<HeosPlayer> {
    let volume = channel.get_volume(&info.pid).await?.level;
    let state = channel.get_play_state(&info.pid).await?.state;
    let now_playing = channel.get_now_playing_media(&info.pid).await?;
    let mode = Some(channel.get_play_mode(&info.pid).await?.mode);

    Ok(HeosPlayer {
        player_id: info.pid,
        name: info.name,
        volume,
        now_playing,
        mode,
        play_state: state,
        in_group: info.gid,
    })
}
