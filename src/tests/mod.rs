//! Utility functions shared among various unit tests.

use crate::*;
use std::env;
use std::path;

mod audio;
mod conf;
mod filesystem;
mod graphics;
mod text;

/// Make a basic `Context` with sane defaults.
pub fn make_context() -> (Context, event::EventsLoop) {
    let mut cb = ContextBuilder::new("ggez_unit_tests", "ggez");
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        cb = cb.add_resource_path(path);
    }
    cb.build().unwrap()
}
