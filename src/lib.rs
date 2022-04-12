#[macro_use]
extern crate serde_derive;
// extern crate futures;
extern crate serde_json;
extern crate serde_qs as qs;

mod error;
pub mod model;
mod connection;
pub mod api;

pub use error::*;
use crate::api::{HeosApi, ApiCommand};

pub type HeosResult<T> = Result<T, HeosError>;

pub async fn create_api() -> HeosResult<HeosApi>{
    println!("Connecting");
    let mut connection = connection::Connection::find().await?;
    let api = api::HeosApi::connect(connection).await?;
    println!("Conneced");
    Ok(api)
}
