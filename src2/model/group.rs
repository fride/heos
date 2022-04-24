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

#[derive(Debug, Clone)]
pub struct SetGroupResponse {
    pub group_name: String,
    pub group_id: GroupId,
    pub players: Vec<PlayerId>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GroupVolume {
    pub group_id: GroupId,
    pub level: Level,
}

#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq)]
pub struct GroupMute {
    pub group_id: GroupId,
    pub state: OnOrOff,
}
