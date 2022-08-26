use std::sync::{Arc, Mutex};
use crate::connection::CommandResult;
use crate::HeosResult;
use crate::model::{GroupId, PlayerId};

pub type Shared<T> = Arc<Mutex<T>>;

pub struct Player {
    id: PlayerId,
    name: String, // this won't be immutable I guess?
    lineout: Option<u64>,
    ip: Option<String>,
    model: Option<String>,
    network: Option<String>,
    version: Option<String>,
    gid: Option<GroupId>,
    control: Option<String>,
}
