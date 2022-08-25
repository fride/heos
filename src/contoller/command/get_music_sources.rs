


use crate::{Connection, HeosResult};
use crate::api::HeosApi;
use crate::contoller::command::ApiCommand;
use crate::contoller::State;



// https://rust-unofficial.github.io/patterns/patterns/behavioural/command.html
// https://users.rust-lang.org/t/how-to-store-async-function-pointers/40846
#[derive(Debug, Default)]
pub struct GetMusicSources;

impl GetMusicSources {
    pub async fn apply(self, connection: &mut Connection, state: &State) -> HeosResult<()> {
        let music_sources = connection.get_music_sources().await?;
        state.set_music_sources(music_sources);
        Ok(())
    }
}
impl Into<ApiCommand> for GetMusicSources {
    fn into(self) -> ApiCommand {
        ApiCommand::GetMusicSources(self)
    }
}
