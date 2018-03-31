//! How to glyph cache text render in ggez.
extern crate ggez;
extern crate gfx_glyph;
extern crate gfx_device_gl;

use ggez::conf;
use ggez::event::{ self, EventHandler };
use ggez::{ Context, ContextBuilder, GameResult };
use ggez::graphics;

use gfx_glyph::{ GlyphBrush, GlyphBrushBuilder, Section };

struct MainState {
    glyph_brush: GlyphBrush<'static, gfx_device_gl::Resources, gfx_device_gl::Factory>,
    frames: usize,
    text_base: String,
    text_src: Vec<char>,
    window_w: f32,
    window_h: f32,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        // gfx_glyph can't through `ggez::graphics::Font`.
        // must used another ways.
        let font = include_bytes!("../resources/DejaVuSerif.ttf");

        // Set window size.
        let (window_w, window_h) = (ctx.conf.window_mode.width as f32,
                                    ctx.conf.window_mode.height as f32);

        // get gfx-rs Factory object.
        let factory = graphics::get_factory(ctx);

        // Create glyph cache brush.
        let builder = GlyphBrushBuilder::using_font_bytes(font as &[u8]);
        let glyph_brush = builder.build(factory.clone());

        Ok(MainState {
            glyph_brush,
            frames: 0,
            text_base: String::new(),
            text_src: "Lorem ipsum dolor sit amet, ferri simul omittantur eam eu, no debet doming dolorem ius.".chars().collect(),
            window_w,
            window_h,
        })
    }
}

impl EventHandler for MainState {
    fn update(&mut self,
              ctx: &mut Context) -> GameResult<()> {
        self.frames += 1;
        if self.frames % 100 == 0 {
            println!("FPS: {}", ggez::timer::get_fps(ctx));
        }

        // Update typewriter like text message
        {
            let i = self.text_base.chars().count();

            if self.frames % 2 == 0 {
                if i < self.text_src.len() {
                    self.text_base.push(self.text_src[i]);
                } else {
                    self.text_base.clear();
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self,
            ctx: &mut Context) -> GameResult<()> {
        // Do gfx-rs drawing
        {
            let (_factory,
                 _device,
                 mut encoder,
                 depthview,
                 colorview) = graphics::get_gfx_objects(ctx);

            // Almost the same as ggez::graphics::clear()
            encoder.clear(&colorview, [0.03, 0.03, 0.03, 1.0]);

            // not wrap massage
            self.glyph_brush.queue(Section {
                text: "Hello, world",
                scale: gfx_glyph::Scale::uniform(42.0),
                color: [0.9, 0.3, 0.3, 1.0],
                ..Section::default()
            });

            // Tutorial message
            self.glyph_brush.queue(Section {
                screen_position: (0.0, self.window_h * 0.08),
                bounds: (self.window_w,
                         self.window_h),
                text: "Let's resize this window!",
                scale: gfx_glyph::Scale::uniform(42.0),
                color: [0.9, 0.3, 0.3, 1.0],
                ..Section::default()
            });

            // Wrap text
            self.glyph_brush.queue(Section {
                screen_position: (0.0, self.window_h * 0.3),
                bounds: (self.window_w,
                         self.window_h),
                text: "The quick brown fox jumps over the lazy dog",
                scale: gfx_glyph::Scale::uniform(36.0),
                color: [0.3, 0.3, 0.9, 1.0],
                ..Section::default()
            });

            // Like a typewriter
            self.glyph_brush.queue(Section {
                screen_position: (0.0, self.window_h * 0.6),
                bounds: (self.window_w,
                         self.window_h),
                text: &self.text_base,
                scale: gfx_glyph::Scale::uniform(36.0),
                color: [0.3, 0.9, 0.3, 1.0],
                ..Section::default()
            });

            // Instead of ggez::graphics::draw()
            self.glyph_brush
                .draw_queued(&mut encoder, &colorview, &depthview)
                .expect("glyph_brush error");
        }

        // Do ggez drawing
        graphics::present(ctx);

        Ok(())
    }

    fn resize_event(&mut self,
                    _ctx: &mut Context,
                    width: u32,
                    height: u32) {
        println!("Resized screen to {}, {}", width, height);
        // reset window size
        self.window_w = width as f32;
        self.window_h = height as f32;
    }
}

/// In default, ggez has create `user_config_dir` and `user_data_dir`.
/// e.g. `$HOME/.config/foobar` and `$HOME/.local/share/foobar`
///
/// but this example don't use these directory.
///
/// "Cast no dirt into the well that gives you water".
fn unused_dir_remove(ctx: &mut Context) -> GameResult<()> {
    let user_conf_dir_path = ctx.filesystem.get_user_config_dir();
    let user_data_dir_path = ctx.filesystem.get_user_data_dir();

    if user_conf_dir_path.is_dir() {
        ::std::fs::remove_dir(user_conf_dir_path)?;
    }

    if user_data_dir_path.is_dir() {
        ::std::fs::remove_dir(user_data_dir_path)?;
    }

    Ok(())
}

pub fn main() {
    let cb = ContextBuilder::new("glyph_cache_example", "ggez")
        .window_setup(conf::WindowSetup::default()
            .title("glyph cache example")
            .resizable(true))
        .window_mode(conf::WindowMode::default()
            .dimensions(512, 512));

    let ctx = &mut cb.build().unwrap();

    // User directory clean up.
    let _ = unused_dir_remove(ctx);

    let state = &mut MainState::new(ctx).unwrap();
    if let Err(e) = event::run(ctx, state) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
