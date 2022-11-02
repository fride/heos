use crate::api::HeosApi;
use crate::contoller::command::{ApiCommand, GetMusicSources, GetPlayers};
use crate::contoller::State;
use crate::model::browse::MusicSource;
use crate::model::player::PlayerInfo;
use crate::{Connection, HeosError, HeosResult};
use syn::token::In;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Receiver;

#[derive(Debug)]
pub struct InitController;

impl InitController {
    pub async fn apply(self, connection: &mut Connection, state: &State) -> HeosResult<()> {
        let _ = GetMusicSources::default().apply(connection, state).await;
        let _ = GetPlayers::default().apply(connection, state).await;
        Ok(())
    }
}

impl Into<ApiCommand> for InitController {
    fn into(self) -> ApiCommand {
        ApiCommand::InitController(self)
    }
}
