use crate::api::HeosApi;
use crate::contoller::command::ApiCommand;
use crate::contoller::State;
use crate::model::player::PlayState;
use crate::model::PlayerId;
use crate::{Connection, HeosResult};

#[derive(Debug)]
pub struct SetPlayState {
    pub player_id: PlayerId,
    pub state: PlayState,
}

impl SetPlayState {
    pub fn stop(player_id: PlayerId) -> Self {
        Self {
            player_id,
            state: PlayState::Stop,
        }
    }

    pub async fn apply(self, connection: &mut Connection, state: &State) -> HeosResult<()> {
        let new_play_state = connection
            .set_play_state(self.player_id, self.state)
            .await?;
        state.set_player_state(new_play_state);
        Ok(())
    }
}
impl Into<ApiCommand> for SetPlayState {
    fn into(self) -> ApiCommand {
        ApiCommand::SetPlayState(self)
    }
}
