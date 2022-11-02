use std::convert::TryFrom;
use std::iter::Scan;

use anyhow::Context;
use qs;
use serde_json::Value;

use crate::connection::{CommandResponse, EventResponse};
use crate::error::HeosError;
use crate::macros::*;
use crate::types::browse::*;
use crate::types::event::*;
use crate::types::group::{CreateGroupResponse, DeleteGroupResponse, GroupInfo, GroupVolume};
use crate::types::player::*;
use crate::types::system::*;
use crate::types::*;
use crate::HeosResult;

jason_parser!(Vec<PlayerInfo>);
jason_parser!(PlayerInfo);
jason_parser!(RegisteredForChangeEvents);
jason_parser!(Vec<MusicSource>);
jason_parser!(Vec<GroupInfo>);
jason_parser!(Vec<QueueEntry>);
json_option_parser!(NowPlayingMedia);

qs_parser!(PlayerPlayState);
qs_parser!(PlayerVolume);
qs_parser!(PlayerMute);
qs_parser!(PlayerPlayMode);
qs_parser!(GroupVolume);
qs_parser!(CreateGroupResponse);
qs_parser!(DeleteGroupResponse);

impl TryFrom<CommandResponse> for Success {
    type Error = HeosError;

    fn try_from(value: CommandResponse) -> Result<Self, Self::Error> {
        Ok(Success)
    }
}

// event parsing!
pub fn response_to_event(response: EventResponse) -> crate::HeosResult<HeosEvent> {
    use anyhow::Context;
    let json = qs_to_json(&response.event_name, &response.message)?;
    let event: HeosEvent = serde_json::from_value(json).with_context(|| {
        format!(
            "failed to handle event `{}`, qs: `{}`",
            &response.event_name, &response.message
        )
    })?;
    Ok(event)
}

// this is used to collect all possible paameters in heos strange query string format
// This may be a bit wasteful but it saves a lot of code.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Default)]
struct EventQueryParams {
    pid: Option<i64>,
    gid: Option<i64>,
    level: Option<u8>,
    mute: Option<OnOrOff>,
    shuffle: Option<OnOrOff>,
    repeat: Option<Repeat>,
    un: Option<String>,
    error: Option<String>,
    cur_pos: Option<Milliseconds>,
    duration: Option<Milliseconds>,
    state: Option<PlayState>,
}

fn qs_to_json(event_name: &str, message: &str) -> crate::HeosResult<serde_json::Value> {
    use serde_json::*;
    let params: Option<EventQueryParams> = if message.is_empty() {
        None
    } else {
        let result = qs::from_str(message).context("Could not parse heos message as qs string");
        Some(result?)
    };
    Ok(json!({ event_name: params }))
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use super::*;

    #[test]
    pub fn test_play_mode() {
        let response: CommandResponse = CommandResponse {
            command_name: "player/get_play_mode".to_string(),
            message: "pid=10&repeat=on_all&shuffle=off".to_string(),
            payload: Default::default(),
            options: Default::default(),
        };
        let play_mode: PlayerPlayMode = response.try_into().unwrap();
    }
}
