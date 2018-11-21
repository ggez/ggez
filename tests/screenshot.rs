
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
fn image_encode() {
    let (c, _e) = &mut make_context();
    let image = graphics::Image::new(c, "/player.png").unwrap();
    image.encode(c, graphics::ImageFormat::Png, "/encode_test.png").unwrap();
}


#[test]
fn save_screenshot() {
    let (c, _e) = &mut make_context();
    let screenshot = graphics::screenshot(c).unwrap();
    // screenshot.encode(c, graphics::ImageFormat::Png, "/screenshot_test.png").unwrap();
}

#[test]
fn load_images() {
    let (c, _e) = &mut make_context();
    let image = graphics::Image::new(c, "/player.png").unwrap();
    // TODO: Add more images, figure out how to store them more nicely
}