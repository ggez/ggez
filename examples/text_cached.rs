//! This example demonstrates how to use `TextCached` to draw TrueType font texts efficiently.
//! Powered by `gfx_glyph` crate.

extern crate ggez;
extern crate rand;

use ggez::conf::{WindowMode, WindowSetup};
use ggez::event;
use ggez::{Context, ContextBuilder, GameResult};
use ggez::graphics::{self, Color, DrawParam, Point2, Scale, TextCached, TextFragment};
use ggez::timer;
use std::env;
use std::path;

struct MainState {
    anima: f32,
    text: TextCached,
    text_too: TextCached,
    fps_display: TextCached,
    chroma: TextCached,
    wonky: TextCached,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let text = TextCached::new(
            ctx,
            TextFragment {
                text: "Hello".to_string(),
                color: Some(Color::new(1.0, 0.0, 0.0, 1.0)),
                scale: Some(Scale::uniform(30.0)),
                ..Default::default()
            },
        )?;

        let text_too = TextCached::new(ctx, ("World!", Color::new(0.0, 1.0, 1.0, 1.0)))?;

        let fps_display = TextCached::new(ctx, "FPS!")?;

        let chroma_string = "Not quite a rainbow";
        let mut chroma = TextCached::new_empty(ctx)?;
        for ch in chroma_string.chars() {
            chroma.add_fragment((
                ch.to_string(),
                Color::new(
                    rand::random::<f32>(),
                    rand::random::<f32>(),
                    rand::random::<f32>(),
                    1.0,
                ),
            ));
        }

        let wonky_string = "So, so wonky.";
        let mut wonky = TextCached::new_empty(ctx)?;
        for ch in wonky_string.chars() {
            wonky.add_fragment(TextFragment {
                text: ch.to_string(),
                scale: Some(Scale::uniform(10.0 + 24.0 * rand::random::<f32>())),
                ..Default::default()
            });
        }

        Ok(MainState {
            anima: 0.0,
            text,
            text_too,
            fps_display,
            chroma,
            wonky,
        })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_FPS: u32 = 60;
        while timer::check_update_time(ctx, DESIRED_FPS) {
            self.anima += 0.02;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        let fps = timer::get_fps(ctx);
        self.fps_display = TextCached::new(ctx, format!("FPS: {}", fps))?;

        graphics::draw_ex(
            ctx,
            &self.text,
            DrawParam {
                dest: Point2::new(200.0, 250.0),
                rotation: self.anima,
                offset: Point2::new(-20.0, -8.0),
                ..Default::default()
            },
        )?;
        graphics::draw_ex(
            ctx,
            &self.text_too,
            DrawParam {
                dest: Point2::new(400.0, 250.0),
                shear: Point2::new(0.0, self.anima.sin()),
                ..Default::default()
            },
        )?;

        self.fps_display.queue(ctx, Point2::new(0.0, 0.0), None);
        self.chroma.queue(ctx, Point2::new(50.0, 50.0), None);
        self.wonky.queue(ctx, Point2::new(50.0, 450.0), None);
        TextCached::draw_queued(ctx, DrawParam::default())?;

        let wobble_string = "WOBBLE";
        let mut wobble = TextCached::new_empty(ctx)?;
        for ch in wobble_string.chars() {
            wobble.add_fragment(TextFragment {
                text: ch.to_string(),
                scale: Some(Scale::uniform(10.0 + 6.0 * rand::random::<f32>())),
                ..Default::default()
            });
        }
        let wobble_offset = Point2::new(0.0, 0.0);
        let wobble_width = wobble.width(ctx);
        let wobble_height = wobble.height(ctx);
        wobble.queue(ctx, wobble_offset, None);
        TextCached::new(
            ctx,
            format!("width: {}\nheight: {}", wobble_width, wobble_height),
        )?.queue(ctx, Point2::new(0.0, 20.0), None);
        TextCached::draw_queued(ctx, (Point2::new(50.0, 400.0), 0.0))?;

        TextCached::new(ctx, "word1")?.queue(
            ctx,
            Point2::new(-50.0, 5.0),
            Some(Color::new(
                rand::random::<f32>(),
                rand::random::<f32>(),
                rand::random::<f32>(),
                1.0,
            )),
        );
        TextCached::new(ctx, "word2")?.queue(
            ctx,
            Point2::new(0.0, -5.0),
            Some(Color::new(
                rand::random::<f32>(),
                rand::random::<f32>(),
                rand::random::<f32>(),
                1.0,
            )),
        );
        TextCached::new(ctx, "word3")?.queue(
            ctx,
            Point2::new(50.0, 0.0),
            Some(Color::new(
                rand::random::<f32>(),
                rand::random::<f32>(),
                rand::random::<f32>(),
                1.0,
            )),
        );
        TextCached::draw_queued(
            ctx,
            DrawParam {
                dest: Point2::new(600.0, 300.0),
                rotation: 0.3,
                shear: Point2::new(0.5, 0.0),
                ..Default::default()
            },
        )?;

        TextCached::new_empty(ctx)?
            .set_font(FontId::default(), Scale::uniform(8.0))
            .set_bounds(Point2::new(100.0, 100.0), None)
            .add_fragment("simple fragment ")
            .add_fragment(("always yellow fragment", Color::new(1.0, 1.0, 0.0, 1.0)))
            .add_fragment(" another simple fragment")
            .queue(ctx, Point2::origin(), Some(Color::new(1.0, 0.0, 0.0, 1.0)));
        TextCached::draw_queued(
            ctx,
            DrawParam {
                dest: Point2::new(500.0, 400.0),
                scale: Point2::new(2.0, 2.0),
                ..Default::default()
            },
        )?;

        graphics::present(ctx);
        timer::yield_now();
        Ok(())
    }

    fn resize_event(&mut self, ctx: &mut Context, width: u32, height: u32) {
        graphics::set_screen_coordinates(
            ctx,
            graphics::Rect::new(0.0, 0.0, width as f32, height as f32),
        ).unwrap();
    }
}

pub fn main() {
    let ctx = &mut ContextBuilder::new("text_cached", "ggez")
        .window_setup(
            WindowSetup::default()
                .title("Cached text example!")
                .resizable(true),
        )
        .window_mode(WindowMode::default().dimensions(640, 480))
        .build()
        .unwrap();

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        ctx.filesystem.mount(&path, true);
    }

    let state = &mut MainState::new(ctx).unwrap();
    if let Err(e) = event::run(ctx, state) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
