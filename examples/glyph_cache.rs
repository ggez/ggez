//! How to glyph cache text render in ggez.
extern crate ggez;

use ggez::conf;
use ggez::event::{self, EventHandler};
use ggez::graphics::glyphcache::{GlyphCache, HorizontalAlign, Layout, TextParam};
use ggez::graphics::{self, Color, Point2};
use ggez::{Context, ContextBuilder, GameResult};

struct MainState {
    frames: usize,
    glyph_brush: GlyphCache<'static>,
    text_base: String,
    text_src: Vec<char>,
    window_w: f32,
    window_h: f32,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        // Set window size.
        let (window_w, window_h) = (
            ctx.conf.window_mode.width as f32,
            ctx.conf.window_mode.height as f32,
        );

        // currently `GlyphCache` no compatible `graphics::Font`
        let font = include_bytes!("../resources/DejaVuSerif.ttf");

        // build glyph brush
        let glyph_brush = GlyphCache::from_bytes(ctx, font as &[u8]);

        // Set background color.
        let bg_color = graphics::Color::from_rgba(50, 50, 50, 255);
        graphics::set_background_color(ctx, bg_color);

        Ok(MainState {
            frames: 0,
            text_base: String::new(),
            text_src: "Lorem ipsum dolor sit amet, ferri simul omittantur eam eu, no debet doming dolorem ius.".chars().collect(),
            glyph_brush,
            window_w,
            window_h,
        })
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
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

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        // glyph cache queue
        {
            // Default setting except font_size and color.
            self.glyph_brush.queue(TextParam {
                text: "Hello, world.",
                font_size: 42.0,
                color: Color::new(0.9, 0.9, 0.9, 1.0),
                ..TextParam::default()
            });

            // Center align(no bounds)
            self.glyph_brush.queue(TextParam {
                text: "Let's resize this window!",
                position: Point2::new(self.window_w / 2.0, self.window_h * 0.1),
                font_size: 42.0,
                color: Color::new(0.9, 0.9, 0.3, 1.0),
                layout: Layout::default().h_align(HorizontalAlign::Center),
                ..TextParam::default()
            });

            // Divided into three parts(left)
            self.glyph_brush.queue(TextParam {
                text: "The quick brown fox jumps over the lazy dog",
                position: Point2::new(0.0, self.window_h * 0.25),
                bounds: Point2::new(self.window_w / 3.15, self.window_h),
                font_size: 24.0,
                color: Color::new(0.3, 0.3, 0.9, 1.0),
                ..TextParam::default()
            });

            // Divided into three parts(center)
            self.glyph_brush.queue(TextParam {
                text: "The quick brown fox jumps over the lazy dog",
                position: Point2::new(self.window_w / 2.0, self.window_h * 0.25),
                bounds: Point2::new(self.window_w / 3.15, self.window_h),
                font_size: 24.0,
                color: Color::new(0.3, 0.3, 0.9, 1.0),
                layout: Layout::default().h_align(HorizontalAlign::Center),
                ..TextParam::default()
            });

            // Divided into three parts(right)
            self.glyph_brush.queue(TextParam {
                text: "The quick brown fox jumps over the lazy dog",
                position: Point2::new(self.window_w, self.window_h * 0.25),
                bounds: Point2::new(self.window_w / 3.15, self.window_h),
                font_size: 24.0,
                color: Color::new(0.3, 0.3, 0.9, 1.0),
                layout: Layout::default().h_align(HorizontalAlign::Right),
                ..TextParam::default()
            });

            // Multi line text
            self.glyph_brush.queue(TextParam {
                text: "1. Multi line test text\n2. Multi line test text\n3. Multi line test text",
                position: Point2::new(0.0, self.window_h * 0.5),
                bounds: Point2::new(self.window_w, self.window_h),
                font_size: 24.0,
                color: Color::new(0.9, 0.3, 0.3, 1.0),
                layout: Layout::default_wrap(),
                ..TextParam::default()
            });

            // Like a typewriter
            self.glyph_brush.queue(TextParam {
                text: &self.text_base,
                position: Point2::new(0.0, self.window_h * 0.7),
                bounds: Point2::new(self.window_w, self.window_h),
                font_size: 36.0,
                color: Color::new(0.3, 0.9, 0.3, 1.0),
                ..TextParam::default()
            });

            // Draws all queue
            self.glyph_brush.draw(ctx)?;
        } // end glyph cache closure

        graphics::present(ctx);

        Ok(())
    }

    fn resize_event(&mut self, _ctx: &mut Context, width: u32, height: u32) {
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
    let mut cb = ContextBuilder::new("glyph_cache_example", "ggez")
        .window_setup(
            conf::WindowSetup::default()
                .title("glyph cache example")
                .resizable(true),
        )
        .window_mode(conf::WindowMode::default().dimensions(512, 512));

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = std::path::PathBuf::from(manifest_dir);
        path.push("resources");
        cb = cb.add_resource_path(path);
    } else {
        println!("Not building from cargo?  Ok.");
    }

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
