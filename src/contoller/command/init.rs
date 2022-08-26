use crate::{Connection, HeosResult};

use crate::contoller::command::{ApiCommand, GetGroups, GetMusicSources, GetPlayers};
use crate::contoller::State;

#[derive(Debug)]
pub struct InitController;

impl InitController {
    pub async fn apply(self, connection: &mut Connection, state: &State) -> HeosResult<()> {
        let _ = GetMusicSources::default().apply(connection, state).await;
        let _ = GetPlayers::default().apply(connection, state).await;
        let _ = GetGroups::default().apply(connection, state).await;
        Ok(())
    }
}

impl Into<ApiCommand> for InitController {
    fn into(self) -> ApiCommand {
        ApiCommand::InitController(self)
    }
}
