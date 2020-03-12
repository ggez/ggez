use crate::graphics::Color;
use crate::tests;
use crate::*;
use cgmath::Point2;

// use std::path;

#[test]
fn image_encode() {
    let (c, _e) = &mut tests::make_context();
    let image = graphics::Image::new(c, "/player.png").unwrap();
    image
        .encode(c, graphics::ImageFormat::Png, "/encode_test.png")
        .unwrap();
}

fn get_rgba_sample(rgba_buf: &[u8], width: usize, sample_pos: Point2<f32>) -> (u8, u8, u8, u8) {
    (
        rgba_buf[(width * sample_pos.y as usize + sample_pos.x as usize) * 4 + 0],
        rgba_buf[(width * sample_pos.y as usize + sample_pos.x as usize) * 4 + 1],
        rgba_buf[(width * sample_pos.y as usize + sample_pos.x as usize) * 4 + 2],
        rgba_buf[(width * sample_pos.y as usize + sample_pos.x as usize) * 4 + 3],
    )
}

fn save_screenshot_test(c: &mut Context) {
    graphics::clear(c, Color::new(0.1, 0.2, 0.3, 1.0));

    let width = graphics::drawable_size(c).0;
    let height = graphics::drawable_size(c).1;

    let topleft = graphics::DrawParam::new()
        .color(graphics::WHITE)
        .dest(Point2::new(0.0, 0.0));
    let topright = graphics::DrawParam::new()
        .color(Color::new(1.0, 0.0, 0.0, 1.0))
        .dest(Point2::new(width / 2.0, 0.0));
    let bottomleft = graphics::DrawParam::new()
        .color(Color::new(0.0, 1.0, 0.0, 1.0))
        .dest(Point2::new(0.0, height / 2.0));
    let bottomright = graphics::DrawParam::new()
        .color(Color::new(0.0, 0.0, 1.0, 1.0))
        .dest(Point2::new(width / 2.0, height / 2.0));

    let rect = graphics::Mesh::new_rectangle(
        c,
        graphics::DrawMode::fill(),
        graphics::types::Rect {
            x: 0.0,
            y: 0.0,
            w: width / 2.0,
            h: height / 2.0,
        },
        graphics::WHITE,
    )
    .unwrap();

    graphics::draw(c, &rect, topleft).unwrap();
    graphics::draw(c, &rect, topright).unwrap();
    graphics::draw(c, &rect, bottomleft).unwrap();
    graphics::draw(c, &rect, bottomright).unwrap();

    // Don't do graphics::present(c) since calling it once (!) would mean that the result of our draw operation
    // went to the front buffer and the active screen texture is actually empty.
    c.gfx_context.encoder.flush(&mut *c.gfx_context.device);

    let screenshot = graphics::screenshot(c).unwrap();

    // Check if screenshot has right general properties
    assert_eq!(width as u16, screenshot.width);
    assert_eq!(height as u16, screenshot.height);
    assert_eq!(None, screenshot.blend_mode);

    // Image comparision or rendered output is hard, but we *know* that top left should be white.
    // So take a samples in the middle of each rectangle we drew and compare.
    // Note that we only use fully saturated colors to avoid any issues with color spaces.
    let rgba_buf = screenshot.to_rgba8(c).unwrap();
    let half_rect = cgmath::Vector2::new(width / 4.0, height / 4.0);
    let width = width as usize;
    assert_eq!(
        topleft.color.to_rgba(),
        get_rgba_sample(&rgba_buf, width, Point2::from(topleft.dest) + half_rect)
    );
    assert_eq!(
        topright.color.to_rgba(),
        get_rgba_sample(&rgba_buf, width, Point2::from(topright.dest) + half_rect)
    );
    assert_eq!(
        bottomleft.color.to_rgba(),
        get_rgba_sample(&rgba_buf, width, Point2::from(bottomleft.dest) + half_rect)
    );
    assert_eq!(
        bottomright.color.to_rgba(),
        get_rgba_sample(&rgba_buf, width, Point2::from(bottomright.dest) + half_rect)
    );

    // save screenshot (no check, just to see if it doesn't crash)
    screenshot
        .encode(c, graphics::ImageFormat::Png, "/screenshot_test.png")
        .unwrap();
}

#[test]
fn save_screenshot() {
    let (c, _e) = &mut tests::make_context();
    save_screenshot_test(c);
}

// Not supported, see https://github.com/ggez/ggez/issues/751
// #[test]
// fn save_screenshot_with_antialiasing() {
//     let cb = ContextBuilder::new("ggez_unit_tests", "ggez")
//         .window_setup(conf::WindowSetup::default().samples(conf::NumSamples::Eight));
//     let (c, _e) = &mut tests::make_context_from_contextbuilder(cb);
//     save_screenshot_test(c);
// }

#[test]
fn load_images() {
    let (c, _e) = &mut tests::make_context();
    let image = graphics::Image::new(c, "/player.png").unwrap();
    image
        .encode(c, graphics::ImageFormat::Png, "/player_save_test.png")
        .unwrap();
    let _i2 = graphics::Image::new(c, "/player_save_test.png").unwrap();
}

#[test]
fn sanity_check_window_sizes() {
    let (c, e) = &mut tests::make_context();

    // Make sure that window sizes are what we ask for, and not what hidpi gives us.
    let w = c.conf.window_mode.width;
    let h = c.conf.window_mode.height;
    let size = graphics::drawable_size(c);
    assert_eq!(w, size.0);
    assert_eq!(h, size.1);

    let outer_size = graphics::size(c);
    assert!(size.0 <= outer_size.0);
    assert!(size.1 <= outer_size.1);

    // Make sure resizing the window works.
    let w = 100.0;
    let h = 200.0;
    graphics::set_drawable_size(c, w, h).unwrap();
    // ahahaha this apparently REQUIRES a delay between setting
    // the size and it actually altering, at least on Linux X11
    std::thread::sleep(std::time::Duration::from_millis(100));
    // Maybe we need to run the event pump too?  It seems VERY flaky.
    // Sometimes you need one, sometimes you need both...
    e.poll_events(|event| {
        c.process_event(&event);
    });

    let size = graphics::drawable_size(c);
    assert_eq!(w, size.0);
    assert_eq!(h, size.1);
}

/// Ensure that the transform stack applies operations in the correct order.
#[test]
fn test_transform_stack_order() {
    let (ctx, _e) = &mut tests::make_context();
    let p1 = graphics::DrawParam::default();
    let p2 = graphics::DrawParam::default();
    let t1 = p1.to_matrix();
    let t2 = p2.to_matrix();
    graphics::push_transform(ctx, Some(t1));
    graphics::mul_transform(ctx, t2);
    let res = crate::nalgebra::Matrix4::<f32>::from(graphics::transform(ctx));
    let m1: crate::nalgebra::Matrix4<f32> = t1.into();
    let m2: crate::nalgebra::Matrix4<f32> = t2.into();
    assert_eq!(res, m2 * m1);
}
