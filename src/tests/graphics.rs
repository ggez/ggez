use crate::tests;
use crate::*;

// use std::path;

#[test]
fn image_encode() {
    let (c, _e) = &mut tests::make_context();
    let image = graphics::Image::new(c, "/player.png").unwrap();
    image
        .encode(c, graphics::ImageFormat::Png, "/encode_test.png")
        .unwrap();
}

#[test]
fn save_screenshot() {
    let (c, _e) = &mut tests::make_context();
    // TODO: This doesn't work.
    // let screenshot = graphics::screenshot(c).unwrap();
    // screenshot.encode(c, graphics::ImageFormat::Png, "/screenshot_test.png").unwrap();
}

#[test]
fn load_images() {
    let (c, _e) = &mut tests::make_context();
    let image = graphics::Image::new(c, "/player.png").unwrap();
    image
        .encode(c, graphics::ImageFormat::Png, "/player_save_test.png")
        .unwrap();
    // TODO: Add more images, figure out how to store them more nicely
}
