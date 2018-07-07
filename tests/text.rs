
// #[cfg(all(test, has_display))]

extern crate ggez;
use ggez::*;

#[test]
pub fn test_text_width() {
    let ctx = &mut Context::load_from_conf("test", "me", conf::Conf::new()).unwrap();
    let font = graphics::Font::default_font().unwrap();

    let text = "Hello There";

    let expected_width = font.get_width(text);
    let rendered_width = graphics::Text::new(ctx, text, &font).unwrap().width();

    println!("Text: {:?}, expected: {}, rendered: {}", text, expected_width, rendered_width);
    assert_eq!(expected_width as usize, rendered_width as usize);
}