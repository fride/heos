extern crate itertools;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_qs as qs;

use crate::error::HeosError;

// this must be at the top to work! See https://stackoverflow.com/questions/26731243/how-do-i-use-a-macro-across-module-files#31749071
#[macro_use]
pub(crate) mod macros;

mod api;
mod connection;
pub mod error;
pub mod types;
pub type HeosResult<T> = Result<T, HeosError>;

pub use api::HeosApi;

mod driver;

pub use driver::HeosDriver;


