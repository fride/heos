use crate::types::OnOrOff;
use serde::{Deserialize, Deserializer};

use super::{GroupId, Level, PlayerId};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum GroupRole {
    #[serde(rename = "leader")]
    Leader,
    #[serde(rename = "member")]
    Member,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GroupMember {
    pub name: String,
    pub pid: PlayerId,
    pub role: GroupRole,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GroupInfo {
    pub name: String,
    pub gid: GroupId,
    pub players: Vec<GroupMember>,
}
impl GroupInfo {
    pub fn has_member(&self, player_id: PlayerId) -> bool {
        self.players.iter().find(|member| member.pid == player_id).is_some()
    }
    pub fn member_ids(&self) -> Vec<PlayerId> {
        self.players.iter().map(|m| m.pid.clone()).collect()
    }
    pub fn leader(&self) -> &GroupMember {
        for member in &self.players {
            if member.role == GroupRole::Leader {
                return member;
            }
        }
        panic!("No Leader in group");
    }
}
pub struct GroupMembers {
    pub leader: GroupMember,
    pub members: Vec<GroupMember>,
}

impl Into<GroupMembers> for GroupInfo {
    fn into(self) -> GroupMembers {
        let (leader, members): (Vec<GroupMember>, Vec<GroupMember>) =
            self.players.into_iter().partition(|m| m.role == GroupRole::Leader);
        GroupMembers {
            leader: leader.into_iter().next().expect("no leader in group"),
            members,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupResponse {
    pub group_name: Option<String>,
    pub gid: GroupId,
    // this is an atrocity!
    #[serde(rename = "pid")]
    #[serde(deserialize_with = "deserialize_silly_list")]
    pub pids: Vec<PlayerId>,
}

fn deserialize_silly_list<'de, D>(deserializer: D) -> Result<Vec<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let mut ids = vec![];
    for id in s.split(",") {
        ids.push(id.parse().map_err(serde::de::Error::custom)?);
    }
    Ok(ids)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteGroupResponse {
    pub pid: PlayerId,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GroupVolume {
    #[serde(rename = "gid")]
    pub group_id: GroupId,
    pub level: Level,
}

#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq)]
pub struct GroupMute {
    #[serde(rename = "gid")]
    pub group_id: GroupId,
    pub state: OnOrOff,
}

#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq)]
pub struct SetGroup {
    pub leader: PlayerId,
    pub member: Vec<PlayerId>,
}
impl SetGroup {
    pub fn delete_group(leader: PlayerId) -> Self {
        Self { leader, member: vec![] }
    }
}

impl Into<SetGroup> for PlayerId {
    fn into(self) -> SetGroup {
        SetGroup::delete_group(self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Group {
    pub name: String,
    pub gid: GroupId,
    pub volume: Level,
    pub players: Vec<GroupMember>,
}

impl Group {
    pub fn leader(&self) -> Option<&GroupMember> {
        for member in &self.players {
            if member.role == GroupRole::Leader {
                return Some(member);
            }
        }
        None
    }
}
