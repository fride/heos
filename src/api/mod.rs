use async_trait::async_trait;
use std::fmt::Display;

use crate::connection::{CommandResponse, Connection};
use crate::model::group::{GroupInfo, GroupVolume};
use crate::model::player::{
    PlayState, PlayerInfo, PlayerMute, PlayerNowPlayingMedia, PlayerPlayMode, PlayerPlayState,
    PlayerVolume, QueueEntry,
};
use crate::model::{GroupId, Level, OnOrOff, PlayMode, PlayerId, Range};
use crate::{HeosError, HeosResult};

mod parsers;


// pub struct HeosState {
//     pub players: Vec<PlayerInfo>,
//     pub groups: Vec<GroupInfo>,
//     pub player_volumes: Vec<PlayerVolume>,
//     pub group_volumes: Vec<GroupVolume>
//     pub group_volumes: Vec<GroupVolume>
// }

#[async_trait]
pub trait HeosApi {
    async fn load_players(&mut self) -> HeosResult<Vec<PlayerInfo>>;
    async fn get_play_state(&mut self, player_id: PlayerId) -> HeosResult<PlayerPlayState>;
    async fn set_play_state(
        &mut self,
        player_id: PlayerId,
        state: PlayState,
    ) -> HeosResult<PlayerPlayState>;
    async fn get_now_playing_media(
        &mut self,
        player_id: PlayerId,
    ) -> HeosResult<PlayerNowPlayingMedia>;
    async fn get_volume(&mut self, player_id: PlayerId) -> HeosResult<PlayerVolume>;
    async fn set_volume(&mut self, player_id: PlayerId, level: Level) -> HeosResult<PlayerVolume>;
    async fn get_mute(&mut self, player_id: PlayerId) -> HeosResult<PlayerMute>;
    async fn set_mute(&mut self, player_id: PlayerId, state: OnOrOff) -> HeosResult<PlayerMute>;
    async fn get_play_mode(&mut self, player_id: PlayerId) -> HeosResult<PlayerPlayMode>;
    async fn set_play_mode(
        &mut self,
        player_id: PlayerId,
        mode: PlayMode,
    ) -> HeosResult<PlayerPlayMode>;
    async fn get_queue(&mut self, player_id: PlayerId, range: Range)
        -> HeosResult<Vec<QueueEntry>>;

    async fn get_groups(&mut self) -> HeosResult<Vec<GroupInfo>>;
    async fn set_group(&mut self, players: Vec<PlayerId>) -> HeosResult<Vec<GroupInfo>>;

    async fn get_group_volume(&mut self, group_id: GroupId) -> HeosResult<GroupVolume>;
    async fn set_group_volume(
        &mut self,
        group_id: GroupId,
        level: Level,
    ) -> HeosResult<GroupVolume>;

    // loads all intersting stuff in one run.
    //
    async fn load(
        &mut self,
    ) -> HeosResult<(Vec<PlayerInfo>, Vec<GroupInfo>, Vec<PlayerNowPlayingMedia>)> {
        let players: Vec<PlayerInfo> = self.load_players().await?;
        let groups: Vec<GroupInfo> = self.get_groups().await?;
        let mut player_now_playing = vec![];
        for player in &players {
            let now_playing = self.get_now_playing_media(player.pid).await?;
            player_now_playing.push(now_playing);
        }
        Ok((players, groups, player_now_playing))
    }
}

#[async_trait]
trait CommandExecutor {
    async fn execute_command<A, B>(&mut self, command: A) -> HeosResult<B>
    where
        A: Display + Send,
        B: TryFrom<CommandResponse, Error = HeosError>;
}

#[async_trait]
impl CommandExecutor for Connection {
    async fn execute_command<A, B>(&mut self, command: A) -> HeosResult<B>
    where
        A: Display + Send,
        B: TryFrom<CommandResponse, Error = HeosError>,
    {
        let _ = self.write_frame(&format!("{}", command)).await?;
        let response = self.read_command_response().await?;
        response.try_into()
    }
}

#[async_trait]
impl HeosApi for Connection {
    async fn load_players(&mut self) -> HeosResult<Vec<PlayerInfo>> {
        self.execute_command("player/get_players").await
    }

    async fn get_play_state(&mut self, player_id: PlayerId) -> HeosResult<PlayerPlayState> {
        self.execute_command(format!("player/get_play_state?pid={pid}", pid = player_id))
            .await
    }
    async fn set_play_state(
        &mut self,
        player_id: PlayerId,
        play_state: PlayState,
    ) -> HeosResult<PlayerPlayState> {
        self.execute_command(format!(
            "player/set_play_state?pid={pid}&state={state}",
            pid = player_id,
            state = play_state
        ))
        .await
    }

    async fn get_now_playing_media(
        &mut self,
        player_id: PlayerId,
    ) -> HeosResult<PlayerNowPlayingMedia> {
        self.execute_command(format!("player/get_now_playing_media?pid={}", player_id))
            .await
    }
    async fn get_volume(&mut self, player_id: PlayerId) -> HeosResult<PlayerVolume> {
        self.execute_command(format!("player/get_volume?pid={}", player_id))
            .await
    }
    async fn set_volume(&mut self, player_id: PlayerId, level: Level) -> HeosResult<PlayerVolume> {
        self.execute_command(format!(
            "player/set_volume?pid={pid}&level={level}",
            pid = player_id,
            level = level
        ))
        .await
    }

    async fn get_mute(&mut self, player_id: PlayerId) -> HeosResult<PlayerMute> {
        self.execute_command(format!("player/get_mute?pid={pid}", pid = player_id))
            .await
    }
    async fn set_mute(&mut self, player_id: PlayerId, state: OnOrOff) -> HeosResult<PlayerMute> {
        self.execute_command(format!(
            "player/set_mute?pid={pid}&state={state}",
            pid = player_id,
            state = state
        ))
        .await
    }
    async fn get_play_mode(&mut self, player_id: PlayerId) -> HeosResult<PlayerPlayMode> {
        self.execute_command(format!("player/get_play_mode?pid={pid}", pid = player_id))
            .await
    }

    async fn set_play_mode(
        &mut self,
        player_id: PlayerId,
        mode: PlayMode,
    ) -> HeosResult<PlayerPlayMode> {
        self.execute_command(format!(
            "player/set_play_mode?pid={pid}&repeat={repeat}&shuffle={shuffle}",
            pid = player_id,
            repeat = mode.repeat,
            shuffle = mode.shuffle
        ))
        .await
    }
    async fn get_queue(
        &mut self,
        player_id: PlayerId,
        range: Range,
    ) -> HeosResult<Vec<QueueEntry>> {
        self.execute_command(format!(
            "player/get_queue?pid={pid}&range={start},{end}",
            pid = player_id,
            start = range.start,
            end = range.end
        ))
        .await
    }

    async fn get_groups(&mut self) -> HeosResult<Vec<GroupInfo>> {
        self.execute_command("group/get_groups").await
    }
    async fn set_group(&mut self, players: Vec<PlayerId>) -> HeosResult<Vec<GroupInfo>> {
        let pids = players
            .into_iter()
            .map(|pid| pid.to_string())
            .collect::<Vec<String>>()
            .join(",");
        self.execute_command(format!("group/set_group?pid={pids}", pids = pids))
            .await
    }

    async fn get_group_volume(&mut self, group_id: GroupId) -> HeosResult<GroupVolume> {
        self.execute_command(format!("group/get_volume?gid={}", group_id))
            .await
    }
    async fn set_group_volume(
        &mut self,
        group_id: GroupId,
        level: Level,
    ) -> HeosResult<GroupVolume> {
        self.execute_command(format!(
            "player/set_volume?pid={pid}&level={level}",
            pid = group_id,
            level = level
        ))
        .await
    }
}
