use anyhow::Context;
use qs;
use serde_json::Value;
use std::collections::BTreeMap;
use std::convert::TryFrom;

use crate::connection::{CommandResponse, EventResponse};
use crate::error::HeosError;
use crate::types::browse::*;
use crate::types::event::*;
use crate::types::group::{CreateGroupResponse, DeleteGroupResponse, GroupInfo, GroupVolume};
use crate::types::player::*;
use crate::types::system::*;
use crate::types::*;

jason_parser!(Vec<PlayerInfo>);
jason_parser!(PlayerInfo);
jason_parser!(RegisteredForChangeEvents);
jason_parser!(Vec<MusicSource>);
jason_parser!(Vec<GroupInfo>);
jason_parser!(Vec<BrowsableMedia>);
jason_parser!(Vec<BroseSourceItem>);
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

    fn try_from(_: CommandResponse) -> Result<Self, Self::Error> {
        Ok(Success)
    }
}

impl TryFrom<CommandResponse> for AccountState {
    type Error = HeosError;

    fn try_from(value: CommandResponse) -> Result<Self, Self::Error> {
        let mut params: BTreeMap<String, String> = qs::from_str(&value.message)
            .with_context(|| format!("failed to parse response as login: {}", value.message))?;
        if let Some(un) = params.remove("un") {
            Ok(AccountState::SignedIn(un))
        } else {
            Ok(AccountState::SignedOut)
        }
    }
}

#[derive(Deserialize, Serialize)]
struct BrowseMusicContainerParameters {
    pub sid: SourceId,
    pub cid: ContainerId,
    #[serde(deserialize_with = "range::deserialize")]
    pub range: Range,
    pub count: usize,
    pub returned: usize,
}

mod range {
    use crate::types::Range;
    use serde::Deserialize;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Range, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let range = String::deserialize(deserializer)?;
        let ranges: Vec<&str> = range.split(",").collect();
        let start = ranges[0].parse().map_err(serde::de::Error::custom)?;
        let end = ranges[1].parse().map_err(serde::de::Error::custom)?;
        Ok(Range { start, end })
    }
}

impl TryFrom<CommandResponse> for BrowseMusicContainerResponse {
    type Error = HeosError;

    fn try_from(value: CommandResponse) -> Result<Self, Self::Error> {
        let params: BrowseMusicContainerParameters = qs::from_str(&value.message)
            .with_context(|| format!("failed to parse response: {}", &value.message))?;
        let items = serde_json::from_value(value.payload)
            .with_context(|| format!("failed to parse response: {}", &value.message))?;
        Ok(BrowseMusicContainerResponse {
            sid: 0,
            cid: params.cid,
            range: params.range,
            count: params.count,
            returned: params.returned,
            items,
        })
    }
}
// event parsing!
pub fn response_to_event(response: EventResponse) -> crate::HeosResult<HeosEvent> {
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
    use super::*;
    use crate::connection::Frame;
    use serde_json::json;

    #[test]
    pub fn test_play_mode() {
        let response: CommandResponse = CommandResponse {
            command_name: "player/get_play_mode".to_string(),
            message: "pid=10&repeat=on_all&shuffle=off".to_string(),
            payload: Default::default(),
            options: Default::default(),
        };
        let _play_mode: PlayerPlayMode = response.try_into().unwrap();
    }

    #[test]
    pub fn test_various_browse_responses() {
        let heos_json_response = json!(
            {
              "heos": {
                "command": "browse/browse",
                "result": "success",
                "message": "sid=-1428708007&returned=1&count=1"
              },
              "payload": [
                {
                  "container": "no",
                  "mid": "inputs/aux_in_1",
                  "type": "station",
                  "playable": "yes",
                  "name": "schöne Box - AUX In",
                  "image_url": ""
                },
                {
                    "name": "AVM FRITZ!Mediaserver",
                    "image_uri": "https://production.ws.skyegloup.com:443/media/images/service/logos/musicsource_logo_servers.png",
                    "image_url": "https://production.ws.skyegloup.com:443/media/images/service/logos/musicsource_logo_servers.png",
                    "type": "heos_server",
                    "sid": 1113840301
                },
                {
                  "container": "yes",
                  "type": "container",
                  "cid": "4:cont1:20:0:0:",
                  "playable": "no",
                  "name": "Musik",
                  "image_url": ""
                },
              ]
        });
        let frame: Frame = Frame::from_json(heos_json_response).unwrap();
        if let Frame::Response(_command_response) = frame {
            // let parsed_response :Vec<BroseSourceItem> = command_response.try_into().unwrap();
            // match parsed_response[0] {
            //     BroseSourceItem::HeosServiceOrServer(heos) => {
            //         assert_eq!(heos.)
            //     }
            //     BroseSourceItem::BrowsableMedia(_) => {}
            // }
            // assert_eq!(parsed_response[0], "inputs/aux_in_1");
            // assert_eq!(parsed_response[1].id(), "1113840301");
            // assert_eq!(parsed_response[2].id(), "4:cont1:20:0:0:");
        } else {
            panic!("NOT THE EXPECTED RESULTS")
        }
    }
}
