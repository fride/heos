use async_trait::async_trait;

use crate::api::HeosApi;
use crate::{CommandChannel, GetPlayers, HeosResult};
use crate::model::player::{PlayerInfo, PlayerMute, PlayerPlayMode, PlayerPlayState, PlayerVolume, PlayState, QueueEntry};
use crate::model::{GroupId, Level, OnOrOff, PlayerId, PlayMode, Range};
use crate::model::browse::MusicSource;
use crate::model::group::{GroupInfo, GroupVolume};
use crate::model::zone::NowPlaying;

#[async_trait]
impl HeosApi for CommandChannel {
    async fn get_player_infos(&mut self) -> HeosResult<Vec<PlayerInfo>> {
        self.schedule(GetPlayers::new()).await
    }

    async fn get_play_state(&mut self, player_id: PlayerId) -> HeosResult<PlayerPlayState> {
        todo!()
    }

    async fn set_play_state(&mut self, player_id: PlayerId, state: PlayState) -> HeosResult<PlayerPlayState> {
        todo!()
    }

    async fn get_now_playing_media(&mut self, player_id: PlayerId) -> HeosResult<NowPlaying> {
        todo!()
    }

    async fn get_volume(&mut self, player_id: PlayerId) -> HeosResult<PlayerVolume> {
        todo!()
    }

    async fn set_volume(&mut self, player_id: PlayerId, level: Level) -> HeosResult<PlayerVolume> {
        todo!()
    }

    async fn get_mute(&mut self, player_id: PlayerId) -> HeosResult<PlayerMute> {
        todo!()
    }

    async fn set_mute(&mut self, player_id: PlayerId, state: OnOrOff) -> HeosResult<PlayerMute> {
        todo!()
    }

    async fn get_play_mode(&mut self, player_id: PlayerId) -> HeosResult<PlayerPlayMode> {
        todo!()
    }

    async fn set_play_mode(&mut self, player_id: PlayerId, mode: PlayMode) -> HeosResult<PlayerPlayMode> {
        todo!()
    }

    async fn get_queue(&mut self, player_id: PlayerId, range: Range) -> HeosResult<Vec<QueueEntry>> {
        todo!()
    }

    async fn get_groups(&mut self) -> HeosResult<Vec<GroupInfo>> {
        todo!()
    }

    async fn set_group(&mut self, players: Vec<PlayerId>) -> HeosResult<Vec<GroupInfo>> {
        todo!()
    }

    async fn get_group_volume(&mut self, group_id: GroupId) -> HeosResult<GroupVolume> {
        todo!()
    }

    async fn set_group_volume(&mut self, group_id: GroupId, level: Level) -> HeosResult<GroupVolume> {
        todo!()
    }

    async fn get_music_sources(&mut self) -> HeosResult<Vec<MusicSource>> {
        todo!()
    }
}
