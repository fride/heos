use crate::api::HeosApi;
use crate::contoller::command::ApiCommand;
use crate::contoller::State;
use crate::model::PlayerId;
use crate::{Connection, HeosResult};

// https://rust-unofficial.github.io/patterns/patterns/behavioural/command.html
// https://users.rust-lang.org/t/how-to-store-async-function-pointers/40846
#[derive(Debug, Default)]
pub struct GetNowPlaying {
    pub player_id: PlayerId,
}

impl GetNowPlaying {
    pub async fn apply(self, connection: &mut Connection, state: &State) -> HeosResult<()> {
        let now_playing = connection.get_now_playing_media(self.player_id).await?;
        state.set_now_playing(self.player_id, now_playing);
        Ok(())
    }
}
impl Into<ApiCommand> for GetNowPlaying {
    fn into(self) -> ApiCommand {
        ApiCommand::GetNowPlaying(self)
    }
}
