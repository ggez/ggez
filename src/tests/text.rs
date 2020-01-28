// #[cfg(all(test, has_display))]

use crate::tests;
use crate::*;

#[test]
fn test_calculated_text_width() {
    let (ctx, _ev) = &mut tests::make_context();
    let font = graphics::Font::default();

    let text = graphics::Text::new(("Hello There", font, 24.0));

    let expected_width = text.width(ctx);
    // For now we just test against a known value, since rendering it
    // is odd.
    assert_eq!(expected_width, 123);
    // let rendered_width = graphics::Text::new((text, font, 24)).unwrap().width();

    // println!("Text: {:?}, expected: {}, rendered: {}", text, expected_width, rendered_width);
    // assert_eq!(expected_width as usize, rendered_width as usize);
}

/// Make sure that the "height" of text with ascenders/descenders
/// is the same as text without
#[test]
fn test_calculated_text_height() {
    let (ctx, _ev) = &mut tests::make_context();
    let font = graphics::Font::default();

    let text1 = graphics::Text::new(("strength", font, 24.0));
    let text2 = graphics::Text::new(("moves", font, 24.0));

    let h1 = text1.height(ctx);
    let h2 = text2.height(ctx);
    assert_eq!(h1, h2);
}

#[test]
fn test_monospace_text_is_actually_monospace() {
    let (ctx, _ev) = &mut tests::make_context();
    let font = graphics::Font::new(ctx, "/DejaVuSansMono.ttf").unwrap();

    let text1 = graphics::Text::new(("Hello 1", font, 24.0));
    let text2 = graphics::Text::new(("Hello 2", font, 24.0));
    let text3 = graphics::Text::new(("Hello 3", font, 24.0));
    let text4 = graphics::Text::new(("Hello 4", font, 24.0));

    let width1 = text1.width(ctx);
    let width2 = text3.width(ctx);
    let width3 = text2.width(ctx);
    let width4 = text4.width(ctx);

    assert_eq!(width1, width2);
    assert_eq!(width2, width3);
    assert_eq!(width3, width4);
}
