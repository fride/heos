use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use druid::{AppDelegate, Command, Data, DelegateCtx, Env, Handled, Lens, Selector, Target};
use im;
use crate::model::{Level, Repeat, Shuffle};
use crate::player::{NowPlayingMedia, PlayState};
use crate::{player, PlayerUpdate};
use crate::spielwiese::Ref;


#[derive(Clone, Data, Default)]
pub struct Player {
    pub name: String,
    pub id: i64,
    pub level: Option<Level>,
    pub now_playing_media: Option<NowPlayingMedia>,
    pub state: Option<PlayState>,
    pub shuffle: Option<Shuffle>,
    pub repeat: Option<Repeat>
}

#[derive(Clone, Data, Default, Lens)]
pub struct PlayerGroup{
    name: String,
    id: i64,
    group_volume: Option<Level>,
    players: im::vector::Vector<Ref<Player>>
}

#[derive(Clone, Data, Lens)]
pub struct AppState{
    groups: im::ordmap::OrdMap<i64, PlayerGroup>,
    players: im::ordmap::OrdMap<i64, Ref<Player>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            groups:  im::ordmap::OrdMap::new(),
            players:  im::ordmap::OrdMap::new(),
        }
    }

    pub fn set_players(&mut self, players: &Vec<player::PlayerInfo>) {
        self.players = im::ordmap::OrdMap::from_iter(
            players.iter()
                .map(|p| (p.pid.clone(), Rc::new(RefCell::new(Player{
                    name: p.name.clone(),
                    id: p.pid.clone(),
                    .. Default::default()
                })))));
    }
    pub fn handle_update(&mut self, event: &PlayerUpdate) {
        match event {
            PlayerUpdate::Players(players) => {
                self.set_players(players);
            }
            PlayerUpdate::Groups(groups) => {
                self.groups = im::ordmap::OrdMap::from_iter(groups.into_iter()
                    .map(|g| {
                        let mut players_in_group = im::vector::Vector::new();
                        let players = g.players.iter().map(|member|{
                            if let Some(player) = self.players.get(&member.pid) {
                                players_in_group.push_back(player.clone());
                            }
                        });
                        (g.gid.clone(), PlayerGroup{
                            name: g.name,
                            id: g.gid,
                            players: players_in_group,
                            .. Default::default()
                        })
                    }));
            }
            PlayerUpdate::NowPlaying(now) => {
                if let Some(mut player) = self.players.get_mut(&now.player_id) {
                    let mut player = player.borrow_mut();
                    player.now_playing_media = Some(now.media);
                }
            }
            PlayerUpdate::PlayerVolume(pid, volume) => {
                if let Some(mut player) = self.players.get_mut(&pid) {
                    let mut player = player.borrow_mut();
                    player.level = Some(volume.clone());
                }
            }
            PlayerUpdate::PlayingProgress(_, _, _) => {}
            PlayerUpdate::PlayerPlaybackError(_, _) => {}
            PlayerUpdate::PlayerVolumeChanged(_, _, _) => {}
            PlayerUpdate::MusicSources(_) => {}
        }
    }
}

pub struct MySillyDelegate{

}

impl AppDelegate<AppState> for MySillyDelegate {
    fn command(&mut self, ctx:
    &mut DelegateCtx, target: Target, cmd: &Command, data: &mut AppState, env: &Env) -> Handled {
        if let Some(value) =  cmd.get(super::SET_PLAYERS) {
            data.handle_update(value);
            Handled::Yes
        } else {
            Handled::No
        }
    }
}


impl Data for PlayState {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}

impl Data for Shuffle {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}

impl Data for Repeat {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}
