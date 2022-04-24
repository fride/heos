use super::SourceId;

#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq)]
pub struct MusicSource {
    pub name: String,

    pub image_url: String,

    #[serde(rename = "type")]
    pub source_type: String,

    pub sid: SourceId,

    pub available: String,

    pub service_username: Option<String>,
}
