use crate::model::OnOrOff;

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
}
pub struct GroupMembers {
    pub leader: GroupMember,
    pub members: Vec<GroupMember>,
}

impl Into<GroupMembers> for GroupInfo {
    fn into(self) -> GroupMembers {
        let (leader, members): (Vec<GroupMember>, Vec<GroupMember>) = self
            .players
            .into_iter()
            .partition(|m| m.role == GroupRole::Leader);
        GroupMembers {
            leader: leader.into_iter().next().expect("no leader in group"),
            members,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetGroupResponse {
    pub group_name: String,
    pub group_id: GroupId,
    pub players: Vec<PlayerId>,
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
