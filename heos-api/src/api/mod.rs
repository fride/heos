use std::fmt::Display;
use std::net::SocketAddr;

use tokio::net::ToSocketAddrs;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info};

use parsers::*;

use crate::connection::{CommandResponse, Connection};
use crate::types::browse::{
    BroseSourceItem, BrowseMusicContainerResponse, MusicSource,
};
use crate::types::event::HeosEvent;
use crate::types::group::{GroupInfo, GroupVolume};
use crate::types::player::{
    NowPlayingMedia, PlayState, PlayerInfo, PlayerMute, PlayerPlayMode, PlayerPlayState,
    PlayerVolume, QueueEntry,
};
use crate::types::system::AccountState;
use crate::types::{
    ContainerId, GroupId, Level, OnOrOff, PlayMode, PlayerId, Range, SourceId, Success,
};
use crate::{HeosError, HeosResult};

mod parsers;

struct ApiCommand(String, oneshot::Sender<HeosResult<CommandResponse>>);
impl ApiCommand {
    pub fn new(command: String, responder: oneshot::Sender<HeosResult<CommandResponse>>) -> Self {
        ApiCommand(command, responder)
    }
    async fn execute(self, connection: &mut Connection) {
        let command_response = connection.execute_command(&self.0).await; // TODO!
        info!("received response: {:?}", &command_response);
        let _ = self.1.send(command_response);
    }
}

// using a channel to ensure only one command is executed at once
// Additionally this gives us &mut functions and cheap clone-ability!
#[derive(Clone, Debug)]
pub struct HeosApi(mpsc::Sender<ApiCommand>, SocketAddr);

impl HeosApi {
    pub async fn connect<T: ToSocketAddrs>(addr: T) -> HeosResult<Self> {
        let connection = Connection::connect(addr).await?;
        Ok(HeosApi::new(connection))
    }

    fn new(mut connection: Connection) -> Self {
        let (s, mut r) = mpsc::channel::<ApiCommand>(32);
        let peer_addr = connection.ip_addr().clone();
        // this is the only thread that executes the commands by talking to the heos device.
        tokio::spawn(async move {
            while let Some(command) = r.recv().await {
                let _ = command.execute(&mut connection).await;
            }
        });
        Self(s, peer_addr)
    }
    async fn execute_command<A, B>(&self, command: A) -> HeosResult<B>
    where
        A: Display + Send,
        B: TryFrom<CommandResponse, Error = HeosError>,
    {
        let command = format!("{}", command);
        tracing::debug!("executing command: {}", &command);
        let (s, r) = oneshot::channel();
        let _ = self.0.send(ApiCommand::new(command, s)).await;
        let response = r.await.expect("Failed to receive response")?;
        tracing::debug!("Got Response: {}", &response);
        response.try_into()
    }

    pub async fn login(&self, un: String, pw: String) -> HeosResult<AccountState> {
        let res: AccountState = self
            .execute_command(format!("system/sign_in?un={}&pw={}", un, pw))
            .await?;
        Ok(res)
    }

    pub async fn get_player_infos(&self) -> HeosResult<Vec<PlayerInfo>> {
        self.execute_command("player/get_players").await
    }

    pub async fn get_play_state(&self, player_id: &PlayerId) -> HeosResult<PlayerPlayState> {
        self.execute_command(format!("player/get_play_state?pid={pid}", pid = player_id))
            .await
    }
    pub async fn set_play_state(
        &self,
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
    pub async fn get_now_playing_media(
        &self,
        player_id: &PlayerId,
    ) -> HeosResult<Option<NowPlayingMedia>> {
        self.execute_command(format!("player/get_now_playing_media?pid={}", player_id))
            .await
    }
    pub async fn get_music_sources(&self) -> HeosResult<Vec<MusicSource>> {
        self.execute_command("browse/get_music_sources").await
    }
    pub async fn get_volume(&self, player_id: &PlayerId) -> HeosResult<PlayerVolume> {
        self.execute_command(format!("player/get_volume?pid={}", player_id))
            .await
    }
    pub async fn set_volume(&self, player_id: PlayerId, level: Level) -> HeosResult<PlayerVolume> {
        self.execute_command(format!(
            "player/set_volume?pid={pid}&level={level}",
            pid = player_id,
            level = level
        ))
        .await
    }

    pub async fn get_mute(&self, player_id: PlayerId) -> HeosResult<PlayerMute> {
        self.execute_command(format!("player/get_mute?pid={pid}", pid = player_id))
            .await
    }
    pub async fn set_mute(&self, player_id: PlayerId, state: OnOrOff) -> HeosResult<PlayerMute> {
        self.execute_command(format!(
            "player/set_mute?pid={pid}&state={state}",
            pid = player_id,
            state = state
        ))
        .await
    }
    pub async fn get_play_mode(&self, player_id: &PlayerId) -> HeosResult<PlayerPlayMode> {
        self.execute_command(format!("player/get_play_mode?pid={pid}", pid = player_id))
            .await
    }

    pub async fn set_play_mode(
        &self,
        player_id: &PlayerId,
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
    pub async fn get_queue(
        &self,
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

    pub async fn get_groups(&self) -> HeosResult<Vec<GroupInfo>> {
        self.execute_command("group/get_groups").await
    }
    pub async fn set_group(&self, players: Vec<PlayerId>) -> HeosResult<()> {
        let pids = players
            .into_iter()
            .map(|pid| pid.to_string())
            .collect::<Vec<String>>()
            .join(",");

        let _: Success = self
            .execute_command(format!("group/set_group?pid={pids}", pids = pids))
            .await?;
        Ok(())
    }

    pub async fn get_group_volume(&self, group_id: GroupId) -> HeosResult<GroupVolume> {
        self.execute_command(format!("group/get_volume?gid={}", group_id))
            .await
    }
    pub async fn set_group_volume(
        &self,
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

    pub async fn browse_music_sources(&self, sid: SourceId) -> HeosResult<Vec<BroseSourceItem>> {
        let music_sources = self
            .execute_command(format!("browse/browse?sid={}", sid))
            .await?;
        Ok(music_sources)
    }

    pub async fn browse_music_containers(
        &self,
        sid: &SourceId,
        cid: &ContainerId,
        range: &Range,
    ) -> HeosResult<BrowseMusicContainerResponse> {
        let music_sources = self
            .execute_command(format!(
                "browse/browse?sid={}&cid={}&range={},{}",
                sid, cid, range.start, range.end
            ))
            .await?;
        Ok(music_sources)
    }

    pub async fn events(&self) -> HeosResult<mpsc::Receiver<HeosEvent>> {
        let mut connection = Connection::connect(self.1).await?;
        let _ = connection
            .execute_command("system/register_for_change_events?enable=on")
            .await?;
        let (s, r) = mpsc::channel(64);
        // TODO whenever I do have the time make this so that it only create one connection! ;)
        tokio::spawn(async move {
            loop {
                let event = connection
                    .read_event()
                    .await
                    .and_then(|event| response_to_event(event));
                match event {
                    Ok(event) => {
                        if let Err(_) = s.send(event).await {
                            break;
                        }
                    }
                    Err(e) => {
                        error!("failed to fetch event. {:?}", e);
                        break;
                    }
                }
            }
        });
        Ok(r)
    }
}
