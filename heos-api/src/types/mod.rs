use std::fmt;
use std::fmt::{Display, Formatter};

pub mod browse;
pub mod event;
pub mod group;
pub mod player;
pub mod system;

pub type PlayerId = i64;
pub type GroupId = i64;
pub type QueueId = i64;
pub type SourceId = i64;
pub type AlbumId = String;
pub type MediaId = String;
pub type ContainerId = String;
pub type Level = u8;
pub type Milliseconds = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub start: u16,
    pub end: u16,
}

impl Range {
    pub fn length(&self) -> u16 {
        self.end - self.start
    }

    pub fn previous(&self) -> Option<Self> {
        if self.start == 0 {
            None
        } else if self.length() > self.start {
            Some(Range{
                start: self.start - self.length(),
                end: self.end
            })
        } else {
            Some(Range{
                start: 0,
                end: self.end
            })
        }
    }
    pub fn next(&self) -> Self {
        Range {
            start: self.end,
            end: self.end + self.length()
        }
    }
    pub fn as_query_str(&self) -> String {
        format!("start={}&end={}", self.start, self.end)
    }
}

impl Default for Range {
    fn default() -> Self {
        Range { start: 0, end: 10 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
    hours: u64,
    minutes: u64,
    seconds: u64,
}
impl std::fmt::Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        if self.hours == 0 {
            write!(f, "{}:{}", self.minutes, self.seconds)
        } else {
            write!(f, "{}:{}:{}", self.hours, self.minutes, self.seconds)
        }
    }
}
impl From<Milliseconds> for Time {
    fn from(milliseconds: u64) -> Time {
        let seconds = (milliseconds / 1000) % 60;
        let minutes = (milliseconds / (1000 * 60)) % 60;
        let hours = (milliseconds / (1000 * 60 * 60)) % 24;
        Time {
            hours: hours,
            minutes: minutes,
            seconds: seconds,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum YesOrNo {
    #[serde(rename = "yes")]
    Yes,
    #[serde(rename = "no")]
    No,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Deserialize, Serialize)]
pub enum OnOrOff {
    #[serde(rename = "on")]
    On,
    #[serde(rename = "off")]
    Off,
}

impl fmt::Display for OnOrOff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                &OnOrOff::Off => "off",
                &OnOrOff::On => "on",
            }
        )
    }
}
impl std::str::FromStr for OnOrOff {
    type Err = String;

    fn from_str(string: &str) -> Result<OnOrOff, String> {
        return match string {
            "on" => Ok(OnOrOff::On),
            "off" => Ok(OnOrOff::Off),
            c => Err(format!("can't convert {} to OnOff", c)),
        };
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PlayMode {
    pub shuffle: Shuffle,
    pub repeat: Repeat,
}

impl PlayMode {
    pub fn default() -> PlayMode {
        PlayMode {
            shuffle: Shuffle::Off,
            repeat: Repeat::Off,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Repeat {
    #[serde(rename = "off")]
    Off,
    #[serde(rename = "on_one")]
    OnOne,
    #[serde(rename = "on_all")]
    OnAll,
}
impl fmt::Display for Repeat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Repeat::Off => "off",
                Repeat::OnOne => "on_one",
                Repeat::OnAll => "on_all",
            }
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Shuffle {
    #[serde(rename = "off")]
    Off,
    #[serde(rename = "on")]
    On,
}

impl fmt::Display for Shuffle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Shuffle::Off => "off",
                Shuffle::On => "on",
            }
        )
    }
}


#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum HeosErrorCode {
    UnrecognizedCommand = 1,
    InvalidId = 2,
    WrongNumberOfArguments = 3,
    RequestedDataNotAvailable = 4,
    ResourceCurrentlyNotAvailable = 5,
    InvalidCredentials = 6,
    CommandCouldNotBeExecuted = 7,
    UserNotLoggedIn = 8,
    ParameterOutOfRange = 9,
    UserNotFound = 10,
    InternalError = 11,
    SystemError = 12,
    ProcessingPreviousCommand = 13,
    MediaCantBePlayed = 14,
    OptionNotSupported = 15,
    Unknown = 16,
}

impl Display for HeosErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Success;
