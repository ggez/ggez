//! This example demonstrates how to use `Text` to draw TrueType font texts efficiently.

use ggez::graphics::{self, Color, Text, TextAlign};
use ggez::timer;
use ggez::{
    conf::{WindowMode, WindowSetup},
    graphics::TextLayout,
};
use ggez::{event, graphics::Rect};
use ggez::{Context, ContextBuilder, GameResult};
use glam::Vec2;
use std::collections::BTreeMap;
use std::env;
use std::f32;
use std::path;

/// Creates a random RGB color.
fn random_color(rng: &mut oorandom::Rand32) -> Color {
    Color::new(rng.rand_float(), rng.rand_float(), rng.rand_float(), 1.0)
}

#[derive(Clone)]
struct TextDraw {
    fragments: Vec<Text>,
    bounds: Rect,
    layout: TextLayout,
}

struct App {
    // Doesn't have to be a `BTreeMap`; it's handy if you care about specific elements,
    // want to retrieve them by trivial handles, and have to preserve ordering.
    // Note that there is absolutely *no benefit* to storing text.
    texts: BTreeMap<&'static str, TextDraw>,
    rng: oorandom::Rand32,
}

impl App {
    #[allow(clippy::needless_update)]
    fn new(ctx: &mut Context) -> GameResult<App> {
        let mut texts = BTreeMap::new();

        // We just use a fixed RNG seed for simplicity.
        let mut rng = oorandom::Rand32::new(314159);

        // This is the simplest way to create a drawable text;
        // the color, font, and scale will be default: white, LiberationMono-Regular, 16px unform.
        // Note that you don't even have to load a font: LiberationMono-Regular is baked into `ggez` itself.
        let text = Text::new().text("Hello, World!");
        // Store the text in `App`s map, for drawing in main loop.
        texts.insert(
            "0_hello",
            TextDraw {
                fragments: vec![text],
                bounds: Rect::new(0., 0., f32::INFINITY, f32::INFINITY),
                layout: TextLayout::tl_single_line(),
            },
        );

        let mut text = vec![Text::new()
            .text("Small red fragment")
            .color(Color::new(1.0, 0.0, 0.0, 1.0))
            .size(10.0)];

        // More fragments can be appended at any time.
        text.push(
            Text::new().text(" default fragment, should be long enough to showcase everything"),
        );
        text.push(
            Text::new()
                .text(" magenta fragment")
                .color(Color::new(1.0, 0.0, 1.0, 1.0)),
        );
        text.push(Text::new().text(" another default fragment, to really drive the point home"));

        // This loads a new TrueType font into the context named "Fancy font".
        ctx.gfx.add_font(
            "Fancy font",
            graphics::FontData::from_path(&ctx.fs, "/Tangerine_Regular.ttf")?,
        );

        text.push(
            Text::new()
                .text(" fancy fragment")
                .font("Fancy font")
                .size(25.),
        );
        text.push(Text::new().text(" and a default one, for symmetry"));
        // Store a copy of the built text, retain original for further modifications.
        texts.insert(
            "1_demo_text_1",
            TextDraw {
                fragments: text.clone(),
                bounds: Rect::new(0., 0., f32::INFINITY, f32::INFINITY),
                layout: TextLayout::tl_single_line(),
            },
        );

        let mut text = TextDraw {
            fragments: text,
            bounds: Rect::new(0., 0., f32::INFINITY, f32::INFINITY),
            layout: TextLayout::tl_single_line(),
        };

        // Text can be wrapped by setting it's bounds, in screen coordinates;
        // vertical bound will cut off the extra off the bottom.
        // Alignment and wrapping behaviour within the bounds can be set by `TextLayout`.
        text.bounds = Rect::new(0.0, 0.0, 400.0, f32::INFINITY);
        text.layout = TextLayout::Wrap {
            h_align: TextAlign::Begin,
            v_align: TextAlign::Begin,
        };
        texts.insert("1_demo_text_2", text.clone());

        text.bounds = Rect::new(0.0, 0.0, 500.0, f32::INFINITY);
        text.layout = TextLayout::Wrap {
            h_align: TextAlign::End,
            v_align: TextAlign::Begin,
        };
        texts.insert("1_demo_text_3", text.clone());

        text.fragments
            .iter_mut()
            .for_each(|fragment| fragment.font = "Fancy font".into());
        text.bounds = Rect::new(0.0, 0.0, 300.0, f32::INFINITY);
        text.layout = TextLayout::Wrap {
            h_align: TextAlign::Middle,
            v_align: TextAlign::Begin,
        };
        texts.insert("1_demo_text_4", text);

        // These methods can be combined to easily create a variety of simple effects.
        let chroma_string = "Not quite a rainbow.";
        let mut chroma_text = vec![];
        for ch in chroma_string.chars() {
            chroma_text.push(Text::new().text(ch).color(random_color(&mut rng)));
        }
        texts.insert(
            "2_rainbow",
            TextDraw {
                fragments: chroma_text,
                bounds: Rect::new(0.0, 0.0, f32::INFINITY, f32::INFINITY),
                layout: TextLayout::tl_single_line(),
            },
        );

        let wonky_string = "So, so wonky.";
        let mut wonky_text = vec![];
        for ch in wonky_string.chars() {
            wonky_text.push(Text::new().text(ch).size(10.0 + 24.0 * rng.rand_float()));
        }
        texts.insert(
            "3_wonky",
            TextDraw {
                fragments: wonky_text,
                bounds: Rect::new(0.0, 0.0, f32::INFINITY, f32::INFINITY),
                layout: TextLayout::tl_single_line(),
            },
        );

        Ok(App { texts, rng })
    }
}

impl event::EventHandler<ggez::GameError> for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;
        while ctx.time.check_update_time(DESIRED_FPS) {}
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(&ctx.gfx, Color::from([0.1, 0.2, 0.3, 1.0]));

        // `Text` can be used in "immediate mode", but it's slightly less efficient
        // in most cases, and horrifically less efficient in a few select ones
        // (using `.width()` or `.height()`, for example).
        let fps = ctx.time.fps();
        let fps_display = Text::new()
            .text(format!("FPS: {}", fps))
            .color(Color::WHITE);
        canvas.draw_text(
            &[fps_display],
            Vec2::new(200.0, 0.0),
            0.0,
            TextLayout::tl_single_line(),
            0,
        );

        let mut height = 0.0;
        for text in self.texts.values() {
            let mut bounds = text.bounds;
            bounds.move_to(Vec2::new(20.0, 20.0 + height));
            canvas.draw_bounded_text(&text.fragments, bounds, 0.0, text.layout, 0);
            //height += 20.0 + text.height(ctx) as f32;
            height += 20.0
                + ctx
                    .gfx
                    .measure_bounded_text(&text.fragments, bounds, text.layout)?
                    .h;
        }

        if let Some(text) = self.texts.get_mut("1_demo_text_3") {
            text.fragments[3].color = random_color(&mut self.rng);
        }

        // Another animation example. Note, this is very inefficient as-is.
        let wobble_string = "WOBBLE";
        let mut wobble = vec![];
        for ch in wobble_string.chars() {
            wobble.push(
                Text::new()
                    .text(ch)
                    .size(10.0 + 6.0 * self.rng.rand_float())
                    .color(Color::new(0.0, 1.0, 1.0, 1.0)),
            );
        }
        let wobble_bounds =
            ctx.gfx
                .measure_text(&wobble, Vec2::ZERO, TextLayout::tl_single_line())?;
        let (wobble_width, wobble_height) = (wobble_bounds.w, wobble_bounds.h);
        let origin = Vec2::new(500.0, 300.0);
        canvas.draw_text(&wobble, origin, -0.5, TextLayout::tl_single_line(), 0);
        let t = Text::new().text(format!(
            "width: {}\nheight: {}",
            wobble_width, wobble_height
        ));
        canvas.draw_text(
            &[t],
            origin + Vec2::new(0.0, 20.0),
            -0.5,
            TextLayout::tl_wrap(),
            0,
        );

        canvas.finish(&mut ctx.gfx)?;
        timer::yield_now();
        Ok(())
    }
}

pub fn main() -> GameResult {
    if cfg!(debug_assertions) && env::var("yes_i_really_want_debug_mode").is_err() {
        eprintln!(
            "Note: Release mode will improve performance greatly.\n    \
             e.g. use `cargo run --example text --release`"
        );
    }
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let (mut ctx, events_loop) = ContextBuilder::new("text_cached", "ggez")
        .window_setup(WindowSetup::default().title("Cached text example!"))
        .window_mode(
            WindowMode::default()
                .dimensions(640.0, 480.0)
                .resizable(true),
        )
        .add_resource_path(resource_dir)
        .build()?;

    let state = App::new(&mut ctx)?;
    event::run(ctx, events_loop, state)
}
