use std::fmt::Display;
use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};

use crate::connection::{CommandExecutor, CommandResponse, Connection};
use crate::model::browse::MusicSource;
use crate::model::{GroupId, Level, OnOrOff, PlayMode, PlayerId, Range};
use crate::{HeosError, HeosResult};

use crate::model::group::{GroupInfo, GroupVolume};
use crate::model::player::{
    PlayState, PlayerInfo, PlayerMute, PlayerPlayMode, PlayerPlayState, PlayerVolume, QueueEntry,
};
use crate::model::zone::NowPlaying;

mod parsers;
// mod command;
// pub use command::{CommandChannel};

#[async_trait]
pub trait HeosApi {
    // get infos about all players in the heos system
    async fn get_player_infos(&mut self) -> HeosResult<Vec<PlayerInfo>>;
    // get the play state, pause etc.
    async fn get_play_state(&mut self, player_id: PlayerId) -> HeosResult<PlayerPlayState>;
    async fn set_play_state(
        &mut self,
        player_id: PlayerId,
        state: PlayState,
    ) -> HeosResult<PlayerPlayState>;
    async fn get_now_playing_media(&mut self, player_id: PlayerId) -> HeosResult<NowPlaying>;
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

    // browse
    async fn get_music_sources(&mut self) -> HeosResult<Vec<MusicSource>>;
}
// TODO kill this!
#[async_trait]
impl HeosApi for Connection {
    async fn get_player_infos(&mut self) -> HeosResult<Vec<PlayerInfo>> {
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

    // todo this may return nothing.
    async fn get_now_playing_media(&mut self, player_id: PlayerId) -> HeosResult<NowPlaying> {
        self.execute_command(format!("player/get_now_playing_media?pid={}", player_id))
            .await
    }
    async fn get_music_sources(&mut self) -> HeosResult<Vec<MusicSource>> {
        self.execute_command("browse/get_music_sources").await
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

type Cmd = (String, oneshot::Sender<HeosResult<CommandResponse>>);
pub struct CommandChannel(mpsc::Sender<Cmd>);
impl CommandChannel {
    pub fn new(mut connection: Connection) -> Self {
        let (s,mut r) = mpsc::channel::<Cmd>(32);
        tokio::spawn(async move {
            while let Some((command, responder)) = r.recv().await {
                let _ = connection.write_frame(&command).await; // TODO!
                let command_response = connection.read_command_response().await;
                let _ = responder.send(command_response);
            }
        });
        Self(s)
    }
}


#[async_trait]
impl CommandExecutor for CommandChannel {
    // execute a command by sending.
    async fn execute_command<A, B>(&mut self, command: A) -> HeosResult<B> where A: Display + Send, B: TryFrom<CommandResponse, Error=HeosError> {
        let command = format!("{}", command);
        tracing::debug!("executing command: {}", &command);
        let (s,r) = oneshot::channel();
        let _ = self.0.send((command, s)).await;
        let response = r.await.expect("Failed to receive response")?;
        response.try_into()
    }
}

#[async_trait]
impl HeosApi for CommandChannel {
    async fn get_player_infos(&mut self) -> HeosResult<Vec<PlayerInfo>> {
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

    // todo this may return nothing.
    async fn get_now_playing_media(&mut self, player_id: PlayerId) -> HeosResult<NowPlaying> {
        self.execute_command(format!("player/get_now_playing_media?pid={}", player_id))
            .await
    }
    async fn get_music_sources(&mut self) -> HeosResult<Vec<MusicSource>> {
        self.execute_command("browse/get_music_sources").await
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
