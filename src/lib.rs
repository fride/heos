#[macro_use]
extern crate serde_derive;
// extern crate futures;
extern crate serde_json;
extern crate serde_qs as qs;

pub use error::*;

use crate::api::{ApiCommand, HeosApi};

pub mod api;
mod connection;
mod error;
pub mod model;
mod driver;

pub type HeosResult<T> = Result<T, HeosError>;

pub async fn create_api() -> HeosResult<HeosApi> {
    println!("Connecting");
    let mut connection = connection::Connection::find().await?;
    let api = api::HeosApi::connect(connection).await?;
    println!("Conneced");
    Ok(api)
}
