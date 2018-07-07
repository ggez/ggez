
// #[cfg(all(test, has_display))]

extern crate ggez;
use ggez::*;

fn make_context() -> ggez::Context {
    let mut cb = ContextBuilder::new("ggez_unit_tests", "ggez");
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        cb = cb.add_resource_path(path);
    }

    cb.build().unwrap()
}

#[test]
fn test_calculated_text_width() {
    let ctx = &mut make_context();
    let font = graphics::Font::default_font().unwrap();

    let text = "Hello There";

    let expected_width = font.get_width(text);
    let rendered_width = graphics::Text::new(ctx, text, &font).unwrap().width();

    println!("Text: {:?}, expected: {}, rendered: {}", text, expected_width, rendered_width);
    assert_eq!(expected_width as usize, rendered_width as usize);
}

#[test]
fn test_monospace_text_is_actually_monospace() {
    let ctx = &mut make_context();
    let font = graphics::Font::new(ctx, "/DejaVuSansMono.ttf", 12)?;

    let text1 = "Hello 1";
    let text2 = "Hello 2";
    let text3 = "Hello 3";
    let text4 = "Hello 4";

    let width1 = font.get_width(text1);
    let width2 = font.get_width(text2);
    let width3 = font.get_width(text3);
    let width4 = font.get_width(text4);

    assert_eq!(width1, width2);
    assert_eq!(width2, width3);
    assert_eq!(width3, width4);
}