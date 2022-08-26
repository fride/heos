use crate::api::HeosApi;
use crate::contoller::command::ApiCommand;
use crate::contoller::State;
use crate::model::group::GroupInfo;
use crate::{Connection, HeosResult};

#[derive(Debug, Default)]
pub struct GetGroups;

impl GetGroups {
    pub async fn apply(self, connection: &mut Connection, state: &State) -> HeosResult<()> {
        tracing::info!("fetching all groups.");
        let group_infos: Vec<GroupInfo> = connection.get_groups().await?;
        for group_info in &group_infos {
            let volume = connection.get_group_volume(group_info.gid).await?;
            state.set_group_volume(volume);
        }
        state.set_groups(group_infos);
        tracing::info!("fetched all groups.");
        Ok(())
    }
}
impl Into<ApiCommand> for GetGroups {
    fn into(self) -> ApiCommand {
        ApiCommand::GetGroups(self)
    }
}
