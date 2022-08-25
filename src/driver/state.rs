use std::collections::BTreeMap;
use std::fmt::Display;
use std::ops::Index;

use itertools::Itertools;

use crate::model::group::{GroupInfo, GroupMember, GroupMembers, GroupRole, GroupVolume};
use crate::model::player::{PlayerInfo, PlayerVolume};
use crate::model::PlayerId;
use crate::model::zone::*;

fn remove_if<T, P>(vec: &mut Vec<T>, pred: P) -> Option<T>
where
    P: Fn(&T) -> bool,
{
    match vec.iter().position(|p| pred(p)) {
        None => None,
        Some(index) => Some(vec.remove(index)),
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

#[derive(Debug, Default)]
pub struct DriverState {
    zones: BTreeMap<PlayerId, Zone>,
    player_to_zone: BTreeMap<PlayerId, PlayerId>,
    last_error: Option<String>,
}

impl DriverState {
    pub fn zone_iter(&self) -> impl Iterator<Item = &Zone> {
        self.zones.values()
    }
    pub fn players(&self) -> Vec<&Player> {
        let mut players = vec![];
        for zone in self.zones.values() {
            players.extend(zone.players());
        }
        players
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

    pub fn set_error<A: Display>(&mut self, error: A) {
        self.last_error = Some(format!("{}", error));
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
    pub fn set_group_volume(&mut self, group_volume: GroupVolume) {
        if let Some(zone) = self.zones.get_mut(&group_volume.group_id) {
            zone.set_volume(group_volume.level);
        }
    }

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
