extern crate itertools;
#[macro_use]
extern crate serde_derive;
// extern crate futures;
extern crate serde_json;
extern crate serde_qs as qs;

use tokio::net::ToSocketAddrs;
pub use error::*;
use crate::connection::Connection;

mod api;
pub use api::HeosApi;
use crate::driver::Driver;

pub mod connection;
pub mod driver;
mod error;
pub mod model;
pub mod ui;
pub(crate) mod util;

pub type HeosResult<T> = Result<T, HeosError>;

pub async fn connect<A>(ip: Option<A>) -> HeosResult<Connection>
where
    A: ToSocketAddrs,
{
    match ip {
        Some(a) => connection::Connection::connect(a).await,
        None => connection::Connection::find().await,
    }
}
pub async fn create_api(connection: Connection) -> HeosResult<HeosApi> {
    Ok(HeosApi::new(connection))
}

pub async fn create_driver(connection: Connection) -> HeosResult<Driver> {
    Driver::create(connection).await
}
