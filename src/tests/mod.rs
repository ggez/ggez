//! Utility functions shared among various unit tests.

use crate::*;
use std::env;
use std::path;

mod audio;
mod conf;
mod filesystem;
mod graphics;
mod mesh;
mod text;

pub fn make_context_from_contextbuilder(mut cb: ContextBuilder) -> (Context, event::EventsLoop) {
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        cb = cb.add_resource_path(path);
    }
    cb.build().unwrap()
}

/// Make a basic `Context` with sane defaults.
pub fn make_context() -> (Context, event::EventsLoop) {
    let cb = ContextBuilder::new("ggez_unit_tests", "ggez");
    make_context_from_contextbuilder(cb)
}
