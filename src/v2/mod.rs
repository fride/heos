use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::{Connection, HeosResult};
use crate::api::HeosApi;
use crate::model::group::GroupInfo;
use crate::model::player::PlayerInfo;
use crate::model::PlayerId;
use crate::model::zone::{NowPlaying, Player, PlayingProgress};

pub enum ModelUpdate{
    Player(Vec<PlayerInfo>),
    Groups(Vec<GroupInfo>)
}

async fn load_player(con: &mut Connection, player_info: PlayerInfo) -> HeosResult<Player> {
    let mut player : Player = player_info.into();
    player.volume = Some(con.get_volume(player.id.clone()).await?.level);
    let play_mode = con.get_play_mode(player.id.clone()).await?;
    player.repeat = Some(play_mode.mode.repeat);
    Ok(player)
}

pub async fn load_players(con: &mut Connection) -> HeosResult<BTreeMap<PlayerId, Player>>{
    let player_infos = con.get_player_infos().await?;
    let mut players : BTreeMap<PlayerId, Player> = BTreeMap::new();
    for player_info in player_infos {
        let player = load_player(con, player_info).await?;
        players.insert(player.id.clone(), player);
    }
    Ok(players)
}


pub async fn no_name_yet(mut con: Connection) -> HeosResult<()>{
    let players = load_players(&mut con).await?;

    // thing that can only be fetched from the devices with events:
    // progress
    // playback error
    Ok(())
}


struct NicerApi{
    connection: Connection,
    progress: Mutex<Arc<BTreeMap<PlayerId, PlayingProgress>>>,
    now_playing: Mutex<Arc<BTreeMap<PlayerId, NowPlaying>>>
}

impl NicerApi {
    pub async fn load_player(&mut self, player_info: PlayerInfo) -> HeosResult<Player> {
        let mut player : Player = player_info.into();
        player.volume = Some(self.connection.get_volume(player.id.clone()).await?.level);
        let play_mode = self.connection.get_play_mode(player.id.clone()).await?;
        player.repeat = Some(play_mode.mode.repeat);
        let now_playing = self.connection.get_now_playing_media(player.id.clone()).await?;
        self.now_playing.lock().unwrap().insert(player.id.clone(), now_playing.clone());
        player.now_playing = now_playing;
        player.progress = self.progress.lock().unwrap().get(&player.id).cloned();
        Ok(player)
    }

    pub async fn load_players(&mut self) -> HeosResult<BTreeMap<PlayerId, Player>>{
        let player_infos = self.connection.get_player_infos().await?;
        let mut players : BTreeMap<PlayerId, Player> = BTreeMap::new();
        for player_info in player_infos {
            let player = self.load_player(player_info).await?;
            players.insert(player.id.clone(), player);
        }
        Ok(players)
    }
}
