use tokio_stream::StreamExt; // needed for while let Some(event) = events.next().await
use crate::Connection;

pub struct EventHandler {

}

impl EventHandler {

    pub async fn new(connection: Connection) -> Self{
        tokio::spawn(async move {
            let events = connection.into_event_stream();
            tokio::pin!(events);
            while let Some(event) = events.next().await {

            }
        });
        unimplemented!()
    }
}
