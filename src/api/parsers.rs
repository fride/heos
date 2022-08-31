use std::convert::TryFrom;

use qs;
use regex::Regex;
use serde_json::Value;

use crate::connection::{CommandResponse, EventResponse};
use crate::model::browse::*;
use crate::model::event::*;
use crate::model::group::{GroupInfo, GroupVolume};
use crate::model::player::*;
use crate::model::system::*;
use crate::model::zone::NowPlaying;
use crate::model::*;

impl TryFrom<CommandResponse> for Vec<PlayerInfo> {
    type Error = crate::HeosError;

    fn try_from(value: CommandResponse) -> Result<Self, Self::Error> {
        let players: Vec<PlayerInfo> = serde_json::from_value(value.payload)?;
        Ok(players)
    }
}

impl TryFrom<CommandResponse> for RegisteredForChangeEvents {
    type Error = crate::HeosError;

    fn try_from(value: CommandResponse) -> Result<Self, Self::Error> {
        let res = qs::from_str(&value.message)?;
        Ok(res)
    }
}
impl TryFrom<CommandResponse> for Vec<MusicSource> {
    type Error = crate::HeosError;

    fn try_from(value: CommandResponse) -> Result<Self, Self::Error> {
        let res = serde_json::from_value(value.payload)?;
        Ok(res)
    }
}
impl TryFrom<CommandResponse> for AccountState {
    type Error = crate::HeosError;

    fn try_from(value: CommandResponse) -> Result<Self, Self::Error> {
        let re = Regex::new(r"signed_in&un=([^&]+)").unwrap();
        if let Some(username) = re
            .captures(&value.message)
            .and_then(|caps| caps.get(1).map(|c| c.as_str()))
        {
            Ok(AccountState::SignedIn(username.to_owned()))
        } else {
            Ok(AccountState::SignedOut)
        }
    }
}
impl TryFrom<CommandResponse> for PlayerVolume {
    type Error = crate::HeosError;

    fn try_from(value: CommandResponse) -> Result<Self, Self::Error> {
        let res = qs::from_str(&value.message)?;
        Ok(res)
    }
}

impl TryFrom<EventResponse> for HeosEvent {
    type Error = crate::HeosError;

    fn try_from(value: EventResponse) -> Result<Self, Self::Error> {
        response_to_event(value)
    }
}

impl TryFrom<CommandResponse> for Vec<GroupInfo> {
    type Error = crate::HeosError;

    fn try_from(value: CommandResponse) -> Result<Self, Self::Error> {
        let groups: Vec<GroupInfo> = serde_json::from_value(value.payload)?;
        Ok(groups)
    }
}
impl TryFrom<CommandResponse> for PlayerMute {
    type Error = crate::HeosError;

    fn try_from(value: CommandResponse) -> Result<Self, Self::Error> {
        let mute: PlayerMute = qs::from_str(&value.message)?;
        Ok(mute)
    }
}
impl TryFrom<CommandResponse> for PlayerPlayMode {
    type Error = crate::HeosError;

    fn try_from(value: CommandResponse) -> Result<Self, Self::Error> {
        let mute: PlayerPlayMode = qs::from_str(&value.message)?;
        Ok(mute)
    }
}
impl TryFrom<CommandResponse> for Vec<QueueEntry> {
    type Error = crate::HeosError;

    fn try_from(value: CommandResponse) -> Result<Self, Self::Error> {
        let mute: Vec<QueueEntry> = serde_json::from_value(value.payload)?;
        Ok(mute)
    }
}
impl TryFrom<CommandResponse> for GroupVolume {
    type Error = crate::HeosError;

    fn try_from(value: CommandResponse) -> Result<Self, Self::Error> {
        let mute: GroupVolume = qs::from_str(&value.message)?;
        Ok(mute)
    }
}

impl TryFrom<CommandResponse> for NowPlaying {
    type Error = crate::HeosError;

    fn try_from(value: CommandResponse) -> Result<Self, Self::Error> {
        // let params: EventQueryParams = qs::from_str(value.message.as_str())?;
        match &value.payload {
            Value::Object(keys) if keys.is_empty() => Ok(NowPlaying::Nothing),
            _ => {
                let media: NowPlayingMedia = serde_json::from_value(value.payload)?;
                Ok(media.into())
            }
        }
    }
}

impl TryFrom<CommandResponse> for PlayerPlayState {
    type Error = crate::HeosError;

    fn try_from(value: CommandResponse) -> Result<Self, Self::Error> {
        let params: PlayerPlayState = qs::from_str(value.message.as_str())?;
        Ok(params)
    }
}

// events
fn response_to_event(response: EventResponse) -> crate::HeosResult<HeosEvent> {
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
// This may be a bit wasetfull but it saves a lot of code.
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
        Some(qs::from_str(message)?)
    };
    Ok(json!({ event_name: params }))
}

#[cfg(test)]
mod tests {
    use serde_json::*;

    use super::*;

    #[test]
    fn account_state_signed_out() {
        let response = CommandResponse {
            command_name: "system/check_account".to_owned(),
            message: "signed_out".to_owned(),
            payload: Value::Null,
            options: Value::Null,
        };
        let account_state: AccountState = response.try_into().unwrap();
        assert_eq!(account_state, AccountState::SignedOut);
    }

    #[test]
    fn account_state_signed_in() {
        let response = CommandResponse {
            command_name: "system/check_account".to_owned(),
            message: "signed_in&un=ikke".to_owned(),
            payload: Value::Null,
            options: Value::Null,
        };
        let account_state: AccountState = response.try_into().unwrap();
        assert_eq!(account_state, AccountState::SignedIn("ikke".to_owned()));
    }

    #[test]
    fn event_parser_test() {
        let event = response_to_event(EventResponse {
            event_name: "event/player_volume_changed".to_owned(),
            message: "pid=12&level=22&mute=on".to_owned(),
        })
        .unwrap();

        assert_eq!(
            event,
            HeosEvent::PlayerVolumeChanged {
                player_id: 12,
                level: 22,
                mute: OnOrOff::On
            }
        );
        let event = response_to_event(EventResponse {
            event_name: "event/group_volume_changed".to_owned(),
            message: "gid=-1899423658&level=32&mute=off".to_owned(),
        })
        .unwrap();

        assert_eq!(
            event,
            HeosEvent::GroupVolumeChanged {
                group_id: -1899423658,
                level: 32,
                mute: OnOrOff::Off
            }
        );
    }
}