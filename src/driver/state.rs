use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Formatter};


use serde_json::{json, Value};

use crate::model::{Level, PlayerId};
use crate::model::group::{GroupInfo, GroupRole};
use crate::model::player::{PlayerInfo, PlayState};
use crate::model::zone::NowPlaying;
use crate::util::Shared;


pub struct Player{
    player_id: PlayerId,
    state: Shared<DriverState>
}

impl Player {
    pub fn new(state: Shared<DriverState>, player_id: PlayerId) -> Self {
        Self{
            player_id,
            state
        }
    }

    pub fn name(&self) -> String {
        self.state.with_state(|s| s.players[&self.player_id].name.clone())
    }

    pub fn to_json(&self) -> Value {
        self.state.with_state(|s| {
            let info = &s.players[&self.player_id];
            let volume = s.player_volumes.get(&self.player_id);
            let state = s.player_states.get(&self.player_id);
            let now_playing = s.player_now_playing.get(&self.player_id);
            serde_json::json!({
                "info": info,
                "volume": volume,
                "state": state,
                "now_playing": now_playing
            })
        })
    }
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}", self.to_json())
    }
}
pub struct Zone{
    leader_id: PlayerId,
    member_ids: Vec<PlayerId>,
    state: Shared<DriverState>
}


impl Zone {

    pub fn new(leader_id: PlayerId,
               member_ids: Vec<PlayerId>,
               state: Shared<DriverState>) -> Self {
        Self {
            leader_id,
            member_ids,
            state
        }
    }
    pub fn is_group(&self) -> bool{
        !self.member_ids.is_empty()
    }

    pub fn name(&self) -> String{
        self.state.with_state(|s|{
            if self.is_group(){
                s.groups[&self.leader_id].name.clone()
            } else{
                s.players[&self.leader_id].name.clone()
            }})
    }

    pub fn players(&self) -> Vec<Player> {
        let mut players = vec![];
        players.push(Player{
            player_id: self.leader_id.clone(),
            state: self.state.clone()
        });
        for member in &self.member_ids {
            players.push(Player{
                state: self.state.clone(),
                player_id: member.clone()
            })
        }
        players
    }

    pub fn to_json(&self) -> Value {
        let is_group = self.is_group();
        self.state.with_state(|s| {
            let now_playing = &s.player_now_playing.get(&self.leader_id);
            if is_group {
                let group_info = &s.groups[&self.leader_id];
                let volume = s.group_volumes.get(&self.leader_id);
                let mut members = vec![];
                for member in &group_info.players {
                    members.push(json!({
                        "pid": &member.pid,
                        "volume" : s.player_volumes.get(&member.pid),
                        "name" : &s.players.get(&member.pid).map(|p| p.name.clone()).unwrap_or_default()
                    }));
                }
                json!({
                    "name" : &group_info.name,
                    "volume": &volume,
                    "now_playing": &now_playing,
                    "members": &members
                })
            } else {
                let leader = &s.players[&self.leader_id];
                let volume = s.player_volumes.get(&self.leader_id);
                json!({
                    "name" : &leader.name,
                    "volume": &volume,
                    "now_playing": &now_playing,
                })
            }
        })
    }
}

impl std::fmt::Display for Zone {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let json = self.to_json();
        write!(f, "{}", serde_json::to_string_pretty(&json).unwrap())
    }
}

#[derive(Default)]
pub struct DriverState {
    pub players: BTreeMap<PlayerId, PlayerInfo>,
    pub player_volumes: BTreeMap<PlayerId, Level>,
    pub player_states: BTreeMap<PlayerId, PlayState>,
    pub player_now_playing: BTreeMap<PlayerId, NowPlaying>,
    pub groups: BTreeMap<PlayerId, GroupInfo>,
    pub group_volumes: BTreeMap<PlayerId, Level>,

}

impl DriverState {

    pub fn add_player(&mut self, player: PlayerInfo) {
        self.players.insert(player.pid, player);
    }

    pub fn set_players(&mut self, players: Vec<PlayerInfo>) {
        for player in players {
            let _new_player = self.players.insert(player.pid, player).is_none();
        }
    }
    pub fn set_groups(&mut self, groups: Vec<GroupInfo>) {
        for (_, player) in self.players.iter_mut() {
            player.gid = None
        }
        self.groups.clear();
        for group in groups {
            for member in &group.players {
                if let Some(player) = self.players.get_mut(&member.pid) {
                    player.gid = Some(group.gid);
                }
            }
            self.groups.insert(group.gid, group);
        }
    }
    pub fn set_play_state(&mut self, player_id: PlayerId, state: PlayState) {
        self.player_states.insert(player_id, state);
    }

    pub fn grouped_player_ids(&self) -> Vec<(PlayerId, Vec<PlayerId>)> {
        let mut result = vec![];
        let mut all_player_ids : BTreeSet<PlayerId> = self.players.keys().cloned().collect();
        for (player_id, group_info)  in &self.groups {
            let members : Vec<PlayerId>=  group_info.players.iter()
                .filter_map(|member| match member.role {
                    GroupRole::Leader => None,
                    GroupRole::Member => Some(member.pid.clone())
                }).collect();
            all_player_ids.retain(|pid| !members.contains(pid));
            all_player_ids.remove(&player_id);
            result.push((player_id.clone(), members));
        }
        for pid in all_player_ids {
            result.push((pid, Vec::new()));
        }
        result
    }
}
