use std::collections::BTreeMap;

use std::ops::Index;

use itertools::Itertools;

use crate::model::group::{GroupInfo, GroupMember, GroupMembers, GroupRole};
use crate::model::player::{NowPlayingMedia, PlayState, PlayerInfo, PlayerVolume};
use crate::model::{Level, OnOrOff, PlayerId, Repeat};

fn remove_if<T, P>(vec: &mut Vec<T>, pred: P) -> Option<T>
where
    P: Fn(&T) -> bool,
{
    match vec.iter().position(|p| pred(p)) {
        None => None,
        Some(index) => Some(vec.remove(index)),
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub volume: Option<Level>,
    pub now_playing: Option<NowPlayingMedia>,
    pub state: Option<PlayState>,
    pub repeat: Option<Repeat>,
    pub mute: Option<OnOrOff>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Zone {
    SinglePlayer(Player),
    PlayerGroup {
        leader: Player,
        zone_name: String,
        zone_volume: Option<Level>,
        members: Vec<Player>,
    },
}

impl Zone {
    fn single_player<P: Into<Player>>(player: P) -> Self {
        Zone::SinglePlayer(player.into())
    }
    fn group(leader: Player, members: Vec<Player>) -> Self {
        let zone_name = Zone::zone_name(&leader, &members);
        Zone::PlayerGroup {
            leader,
            zone_name,
            zone_volume: None,
            members,
        }
    }

    pub fn id(&self) -> i64 {
        match self {
            Zone::SinglePlayer(sp) => sp.id.clone(),
            Zone::PlayerGroup { leader, .. } => leader.id.clone(),
        }
    }
    // todo - this is a bit silly ;)
    fn zone_name(leader: &Player, members: &Vec<Player>) -> String {
        format!(
            "{} + {}",
            &leader.name,
            members
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<&str>>()
                .join(" + ")
        )
    }
    pub fn find_player(&self, player_id: PlayerId) -> Option<&Player> {
        match self {
            Zone::SinglePlayer(ref player) => Some(player),
            Zone::PlayerGroup {
                leader, members, ..
            } => {
                if leader.id == player_id {
                    Some(&leader)
                } else {
                    for member in members {
                        if member.id == player_id {
                            return Some(&member);
                        }
                    }
                    None
                }
            }
        }
    }

    pub fn into_players(self) -> Vec<Player> {
        self.into()
    }

    pub fn with_players<A: IntoIterator<Item = Player>>(self, players: A) -> Self {
        match self {
            Zone::SinglePlayer(leader) => {
                let members = players.into_iter().collect();
                let zone_name = Zone::zone_name(&leader, &members);
                Zone::PlayerGroup {
                    leader,
                    zone_volume: None,
                    zone_name,
                    members,
                }
            }
            Zone::PlayerGroup {
                leader,
                zone_name: _zone_name,
                zone_volume,
                mut members,
            } => {
                let new_members = {
                    members.extend(players);
                    members
                };
                let zone_name = Zone::zone_name(&leader, &new_members);
                Zone::PlayerGroup {
                    leader,
                    zone_volume,
                    zone_name,
                    members: new_members,
                }
            }
        }
    }
}

// returns the zone - not the player ;)
impl Index<&PlayerId> for DriverState {
    type Output = Zone;
    fn index(&self, index: &PlayerId) -> &Self::Output {
        let zone_id = &self.player_to_zone[index];
        &self.zones[zone_id]
    }
} // returns the zone - not the player ;)

impl Into<Zone> for PlayerInfo {
    fn into(self) -> Zone {
        match self.gid {
            None => Zone::SinglePlayer(self.into()),
            Some(gid) if gid == self.pid => {
                let leader = self.into();
                let zone_name = Zone::zone_name(&leader, &vec![]);
                Zone::PlayerGroup {
                    leader,
                    zone_volume: None,
                    zone_name,
                    members: vec![],
                }
            }
            Some(gid) => {
                let members = vec![self.into()];
                let leader = Player {
                    id: gid,
                    ..Default::default()
                };
                let zone_name = Zone::zone_name(&leader, &vec![]);
                Zone::PlayerGroup {
                    leader,
                    zone_volume: None,
                    zone_name,
                    members,
                }
            }
        }
    }
}

impl Into<Vec<Player>> for Zone {
    fn into(self) -> Vec<Player> {
        match self {
            Zone::SinglePlayer(player) => vec![player],
            Zone::PlayerGroup {
                leader,
                zone_volume: _,
                mut members,
                ..
            } => {
                members.insert(0, leader);
                members
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct DriverState {
    zones: BTreeMap<PlayerId, Zone>,
    player_to_zone: BTreeMap<PlayerId, PlayerId>,
}

impl DriverState {
    pub fn zone_iter(&self) -> impl Iterator<Item = &Zone> {
        self.zones.values()
    }
    pub fn find_zone(&self, player_id: PlayerId) -> Option<&Zone> {
        self.zones.get(&player_id)
    }

    pub fn find_player(&self, player_id: PlayerId) -> Option<&Player> {
        self.player_to_zone
            .get(&player_id)
            .and_then(|zone_id| self.zones.get(zone_id))
            .and_then(|zone| zone.find_player(player_id))
    }

    pub fn set_groups(&mut self, groups: Vec<GroupInfo>) {
        let mut all_players = self.ungroup_all();
        for group_info in groups {
            let zone = group_info_to_zone(group_info, &mut all_players);
            self.zones.insert(zone.id(), zone);
        }
        for (pid, player) in all_players {
            self.zones.insert(pid, Zone::SinglePlayer(player));
        }
        self.set_player_to_zone();
    }

    pub fn set_players(&mut self, players: Vec<PlayerInfo>) {
        self.zones.clear();
        self.player_to_zone.clear();

        let foo = players
            .into_iter()
            .map(|player| (player.gid.clone(), player))
            .into_group_map();

        for (group_id, mut players) in foo {
            if let Some(gid) = group_id {
                if let Some((index, _)) = players.iter().find_position(|player| player.pid == gid) {
                    let leader = players.remove(index);
                    let zone = Zone::group(
                        leader.into(),
                        players.into_iter().map(|p| p.into()).collect(),
                    );
                    self.zones.insert(zone.id(), zone);
                }
            } else {
                for player in players {
                    self.zones
                        .insert(player.pid.clone(), Zone::SinglePlayer(player.into()));
                }
            }
        }
        self.set_player_to_zone();
    }
    //TODO I don't get the mute handler!?
    pub fn update_player<A>(&mut self, player_id: PlayerId, mut handler: A)
    where
        A: FnMut(&mut Player) -> (),
    {
        if let Some(zone_id) = self.player_to_zone.get(&player_id) {
            if let Some(zone) = self.zones.get_mut(zone_id) {
                match zone {
                    Zone::SinglePlayer(player) => handler(player),
                    Zone::PlayerGroup { leader, .. } if leader.id == player_id => handler(leader),
                    Zone::PlayerGroup { members, .. } => {
                        for member in members {
                            if member.id == player_id {
                                handler(member);
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn set_player_volume(&mut self, player_volume: PlayerVolume) {
        self.update_player(player_volume.player_id, |player| {
            player.volume = Some(player_volume.level)
        });
    }
    fn set_player_to_zone(&mut self) {
        self.player_to_zone.clear();
        for (player_id, zone) in &self.zones {
            match zone {
                Zone::SinglePlayer(_) => {
                    self.player_to_zone
                        .insert(player_id.clone(), player_id.clone());
                }
                Zone::PlayerGroup { members, .. } => {
                    self.player_to_zone
                        .insert(player_id.clone(), player_id.clone());
                    for member in members {
                        self.player_to_zone
                            .insert(member.id.clone(), player_id.clone());
                    }
                }
            };
        }
    }
    fn ungroup_all(&mut self) -> BTreeMap<PlayerId, Player> {
        self.player_to_zone.clear();
        let mut all_players = BTreeMap::new();
        let all_player_ids = self.zones.keys().cloned().collect_vec();
        for player_id in all_player_ids {
            if let Some(zone) = self.zones.remove(&player_id) {
                for player in zone.into_players() {
                    all_players.insert(player.id.clone(), player);
                }
            }
        }
        all_players
    }
}

fn is_leader(member: &GroupMember) -> bool {
    member.role == GroupRole::Leader
}

fn group_info_to_zone(group_info: GroupInfo, all_players: &mut BTreeMap<PlayerId, Player>) -> Zone {
    let group_members: GroupMembers = group_info.into();
    let leader: Player = all_players.remove(&group_members.leader.pid).unwrap();
    let members = group_members
        .members
        .iter()
        .filter_map(|m| all_players.remove(&m.pid))
        .collect::<Vec<Player>>();
    let zone_name = Zone::zone_name(&leader, &members);
    Zone::PlayerGroup {
        leader,
        members,
        zone_volume: None,
        zone_name,
    }
}

impl Into<Player> for PlayerInfo {
    fn into(self) -> Player {
        Player {
            name: self.name,
            id: self.pid,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn zones_are_build_correclty() {
        let players_str = r#"
                    [ {"name": "Heimkino", "pid": 1128532863, "model": "HEOS HomeCinema", "version": "1.583.147", "ip": "192.168.178.34", "network": "wifi", "lineout": 0},
                      {"name": "schöne Box", "pid": -1428708007, "gid": -1899423658, "model": "HEOS 7", "version": "1.583.147", "ip": "192.168.178.35", "network": "wifi", "lineout": 0},
                      {"name": "Küche", "pid": -1899423658, "gid": -1899423658, "model": "HEOS 1", "version": "1.583.147", "ip": "192.168.178.27", "network": "wifi", "lineout": 0}]
                "#;

        let groups: Vec<Vec<GroupInfo>> = {
            let strs = vec![
                json!([
                        {"name": "Küche + schöne Box", "gid": -1899423658,
                            "players": [
                                {"name": "schöne Box", "pid": -1428708007, "role": "member"},
                                {"name": "Küche", "pid": -1899423658, "role": "leader"}]}
                ]),
                json!([
                        {"name": "Küche + Heimkino", "gid": -1899423658,
                            "players": [
                                {"name": "Heimkino", "pid": 1128532863, "role": "member"},
                                {"name": "Küche", "pid": -1899423658, "role": "leader"}
                    ]}
                ]),
                json!(
                  [
                    {"name": "Küche + Heimkino + schöne Box", "gid": -1899423658,
                        "players": [
                            {"name": "Heimkino", "pid": 1128532863, "role": "member"},
                            {"name": "schöne Box", "pid": -1428708007, "role": "member"},
                            {"name": "Küche", "pid": -1899423658, "role": "leader"}
                        ]}
                  ]
                ),
                json!([]),
            ];
            strs.into_iter()
                .map(|s| serde_json::from_value(s).unwrap())
                .collect()
        };

        let player_infos: Vec<PlayerInfo> = serde_json::from_str(players_str).unwrap();

        let mut state = DriverState::default();
        state.set_players(player_infos);
        println!("{:?}", &state);
        for group_infos in groups {
            state.set_groups(group_infos);
            println!("\n\n");
            for zone in state.zone_iter() {
                println!("\t - {:?}", &zone);
            }
        }
    }
}
