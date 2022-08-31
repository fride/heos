use std::fmt;

pub type PlayerId = i64;
pub type GroupId = i64;
pub type QueueId = i64;
pub type SourceId = i64;
pub type AlbumId = String;
pub type MediaId = String;
pub type ContainerId = String;
pub type Level = u8;
pub type Milliseconds = u64;

#[derive(Debug, Clone)]
pub struct Range {
    pub start: u16,
    pub end: u16,
}

impl Default for Range {
    fn default() -> Self {
        Range { start: 0, end: 100 }
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
    Off,
    OnOne,
    OnAll,
}

impl std::str::FromStr for Repeat {
    type Err = String;

    fn from_str(string: &str) -> Result<Repeat, String> {
        return match string {
            "on_all" => Ok(Repeat::OnAll),
            "on_one" => Ok(Repeat::OnOne),
            "off" => Ok(Repeat::Off),
            c => Err(format!("can't convert {} to PlayState", c)),
        };
    }
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
    Off,
    On,
}

impl std::str::FromStr for Shuffle {
    type Err = String;

    fn from_str(string: &str) -> Result<Shuffle, String> {
        return match string {
            "on" => Ok(Shuffle::Off),
            "off" => Ok(Shuffle::On),
            c => Err(format!("can't convert {} to PlayState", c)),
        };
    }
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
