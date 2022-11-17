extern crate maud;

use parking_lot::Mutex;
use std::sync::Arc;

pub type Shared<A> = Arc<Mutex<A>>;

pub fn new_shared<B, A: Into<B>>(value: A) -> Shared<B> {
    Arc::new(Mutex::new(value.into()))
}

pub mod application;
pub mod configuration;
pub mod domain;
pub mod routers;
pub mod telemetry;
pub mod views;
