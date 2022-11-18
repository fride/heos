use heos_api::HeosDriver;

// macros need to go to the top!
#[macro_use]
pub mod axum_ructe;

// The normal style for documenting modules is to place the doc-comments inside the module
// files at the top with `//!`, known as internal doc comments.
//
// However, this style better facilitates a guided exploration of the code, so it's the one
// we'll be using in this project.

/// Defines the arguments required to start the server application using [`clap`].
///
/// [`clap`]: https://github.com/clap-rs/clap/
pub mod config;
pub mod controllers;
pub mod error;
pub mod views;
pub mod models;
#[derive(Clone)]
pub struct ApiContext {
    pub driver: HeosDriver,
}

// include generated templates
include!(concat!(env!("OUT_DIR"), "/templates.rs"));
