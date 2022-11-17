use serde::Deserialize;
use crate::types::{ContainerId, MediaId, YesOrNo};
use super::SourceId;

#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq)]
pub struct MusicSource {
    pub name: String,
    pub image_url: String,
    #[serde(rename = "type")]
    pub source_type: String,
    pub sid: SourceId,
    // of course a string!
    #[serde(deserialize_with = "bool_stringly_typed")]
    pub available: bool,
    pub service_username: Option<String>,
}

pub fn bool_stringly_typed<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s == "true" {
        Ok(true)
    } else {
        Ok(false)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HeosService {
    pub name: String,
    pub sid: SourceId,
    #[serde(rename = "type")]
    pub server_type: String,
    pub image_url: String
}

// browse source can return a lot of different types, for whatever reason!
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum BroseSourceItem {
    HeosService(HeosService),
    BrowsableMedia(BrowsableMedia)
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MediaType {
    #[serde(rename = "artist")]
    Artist,
    #[serde(rename = "album")]
    Album,
    #[serde(rename = "song")]
    Song,
    #[serde(rename = "container")]
    Container,
    #[serde(rename = "station")]
    Station,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BrowsableMedia {
    #[serde(rename = "type")]
    pub media_type: MediaType,
    #[serde(rename = "cid")]
    pub container_id: Option<String>,
    pub playable: YesOrNo,
    pub image_url: String,
    pub name: String,
    pub artist : Option<String>,
    pub album : Option<String>,
    pub mid : Option<MediaId>,
}
