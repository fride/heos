use crate::types::OnOrOff;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum AccountState {
    #[serde(rename = "signed_out")]
    SignedOut,
    #[serde(rename = "signed_in")]
    SignedIn(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct SignedIn {
    #[serde(rename = "un")]
    user_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegisteredForChangeEvents {
    pub enable: OnOrOff,
}
