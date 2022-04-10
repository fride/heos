use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::{HeosError, HeosEvent, HeosResult};
use crate::connection::Connection;

pub async fn heos_event_component(mut connection: Connection, events: Sender<HeosEvent>, errors: Sender<HeosError>) -> HeosResult<()>{
    let _ = connection.write_frame("system/register_for_change_events?enable=on").await?;
    connection.read_command_response().await?;

    tokio::spawn(async move {
        loop {
            let _ = match connection.read_event().await {
                Ok(event_payload) => {
                    let event: HeosResult<HeosEvent> = event_payload.try_into();
                    match event {
                        Ok(event) => { events.send(event).await; }
                        Err(err) => {
                            errors.send(err).await;
                        }
                    }
                }
                Err(err) => {
                    errors.send(err).await;
                }
            };
        }
    });
    Ok(())
}
