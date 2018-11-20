// #[cfg(all(test, has_display))]

extern crate ggez;
use ggez::*;
use std::env;
use std::path;

fn make_context() -> (Context, event::EventsLoop) {
    let mut cb = ContextBuilder::new("ggez_unit_tests", "ggez");
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        cb = cb.add_resource_path(path);
    }
    cb.build().unwrap()
}
/* TODO; the font API has changed and I don't want to deal with it now
#[test]
fn test_calculated_text_width() {
    let ctx = &mut make_context();
    let font = graphics::Font::default();

    let text = "Hello There";

    let expected_width = font.width(text);
    let rendered_width = graphics::Text::new((text, font, 24)).unwrap().width();

    println!("Text: {:?}, expected: {}, rendered: {}", text, expected_width, rendered_width);
    assert_eq!(expected_width as usize, rendered_width as usize);
}

#[test]
fn test_monospace_text_is_actually_monospace() {
    let ctx = &mut make_context();
    let font = graphics::Font::new(ctx, "/DejaVuSansMono.ttf");

    let text1 = "Hello 1";
    let text2 = "Hello 2";
    let text3 = "Hello 3";
    let text4 = "Hello 4";

    let width1 = font.width(text1);
    let width2 = font.width(text2);
    let width3 = font.width(text3);
    let width4 = font.width(text4);

    assert_eq!(width1, width2);
    assert_eq!(width2, width3);
    assert_eq!(width3, width4);
}

*/
