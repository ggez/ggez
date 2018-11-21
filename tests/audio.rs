
extern crate ggez;
use ggez::*;

use std::env;
use std::path;

// TODO: Is there a good way to dedupe this?
fn make_context() -> (Context, event::EventsLoop) {
    let mut cb = ContextBuilder::new("ggez_unit_tests", "ggez");
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        cb = cb.add_resource_path(path);
    }
    cb.build().unwrap()
}

#[test]
fn audio_load() {
    let (c, _e) = &mut make_context();
    {
        // TODO: Test different sound formats
    let mut sound = audio::Source::new(c, "/pew.ogg").unwrap();
    sound.play().unwrap();
    }
    
    // TODO: This is awkward, we should have a way to check whether
    // a file is valid without trying to play it?
    // let mut sound = audio::Source::new(c, "/player.png").unwrap();
    // sound.play().unwrap();
}

