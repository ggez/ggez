//! This example demonstrates how to use `Text` to draw TrueType font texts efficiently.

use cgmath;
use ggez;
use rand;

use cgmath::Point2;
use ggez::conf::{WindowMode, WindowSetup};
use ggez::event;
use ggez::graphics::{self, Align, Color, DrawParam, Font, Scale, Text, TextFragment};
use ggez::timer;
use ggez::{Context, ContextBuilder, GameResult};
use std::collections::BTreeMap;
use std::env;
use std::f32;
use std::path;

/// Creates a random RGB color.
fn random_color() -> Color {
    Color::new(
        rand::random::<f32>(),
        rand::random::<f32>(),
        rand::random::<f32>(),
        1.0,
    )
}

struct App {
    // Doesn't have to be a `BTreeMap`; it's handy if you care about specific elements,
    // want to retrieve them by trivial handles, and have to preserve ordering.
    texts: BTreeMap<&'static str, Text>,
}

impl App {
    fn new(ctx: &mut Context) -> GameResult<App> {
        let mut texts = BTreeMap::new();

        // This is the simplest way to create a drawable text;
        // the color, font, and scale will be default: white, DejaVuSerif, 16px unform.
        // Note that you don't even have to load a font: DejaVuSerif is baked into `ggez` itself.
        let text = Text::new("Hello, World!");
        // Store the text in `App`s map, for drawing in main loop.
        texts.insert("0_hello", text);

        // This is what actually happens in `Text::new()`: the `&str` gets
        // automatically converted into a `TextFragment`.
        let mut text = Text::new(TextFragment {
            // `TextFragment` stores a string, and optional parameters which will override those
            // of `Text` itself. This allows inlining differently formatted lines, words,
            // or even individual letters, into the same block of text.
            text: "Small red fragment".to_string(),
            color: Some(Color::new(1.0, 0.0, 0.0, 1.0)),
            // `Font` is a handle to a loaded TTF, stored inside the `Context`.
            // `Font::default()` always exists and maps to DejaVuSerif.
            font: Some(graphics::Font::default()),
            scale: Some(Scale::uniform(10.0)),
            // This doesn't do anything at this point; can be used to omit fields in declarations.
            ..Default::default()
        });

        // More fragments can be appended at any time.
        text.add(" default fragment, should be long enough to showcase everything")
            // `add()` can be chained, along with most `Text` methods.
            .add(TextFragment::new(" magenta fragment").color(Color::new(1.0, 0.0, 1.0, 1.0)))
            .add(" another default fragment, to really drive the point home");

        // This loads a new TrueType font into the context and
        // returns a `Font` referring to it.
        let fancy_font = Font::new(ctx, "/Tangerine_Regular.ttf")?;

        // `Font` is really only an integer handle, and can be copied around.
        text.add(
            TextFragment::new(" fancy fragment")
                .font(fancy_font)
                .scale(Scale::uniform(25.0)),
        )
        .add(" and a default one, for symmetry");
        // Store a copy of the built text, retain original for further modifications.
        texts.insert("1_demo_text_1", text.clone());

        // Text can be wrapped by setting it's bounds, in screen coordinates;
        // vertical bound will cut off the extra off the bottom.
        // Alignment within the bounds can be set by `Align` enum.
        text.set_bounds(Point2::new(400.0, f32::INFINITY), Align::Left);
        texts.insert("1_demo_text_2", text.clone());

        text.set_bounds(Point2::new(500.0, f32::INFINITY), Align::Right);
        texts.insert("1_demo_text_3", text.clone());

        // This can be used to set the font and scale unformatted fragments will use.
        // Color is specified when drawing (or queueing), via `DrawParam`.
        // Side note: TrueType fonts aren't very consistent between themselves in terms
        // of apparent scale - this font with default scale will appear too small.
        text.set_font(fancy_font.clone(), Scale::uniform(16.0))
            .set_bounds(Point2::new(300.0, f32::INFINITY), Align::Center);
        texts.insert("1_demo_text_4", text);

        // These methods can be combined to easily create a variety of simple effects.
        let chroma_string = "Not quite a rainbow.";
        // `default()` exists pretty much specifically for this usecase.
        let mut chroma_text = Text::default();
        for ch in chroma_string.chars() {
            chroma_text.add(TextFragment::new(ch).color(random_color()));
        }
        texts.insert("2_rainbow", chroma_text);

        let wonky_string = "So, so wonky.";
        let mut wonky_text = Text::default();
        for ch in wonky_string.chars() {
            wonky_text.add(
                TextFragment::new(ch).scale(Scale::uniform(10.0 + 24.0 * rand::random::<f32>())),
            );
        }
        texts.insert("3_wonky", wonky_text);

        Ok(App { texts })
    }
}

impl event::EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;
        while timer::check_update_time(ctx, DESIRED_FPS) {}
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        // `Text` can be used in "immediate mode", but it's slightly less efficient
        // in most cases, and horrifically less efficient in a few select ones
        // (using `.width()` or `.height()`, for example).
        let fps = timer::fps(ctx);
        let fps_display = Text::new(format!("FPS: {}", fps));
        // When drawing through these calls, `DrawParam` will work as they are documented.
        graphics::draw(
            ctx,
            &fps_display,
            (Point2::new(200.0, 0.0), graphics::WHITE),
        )?;

        let mut height = 0.0;
        for (_key, text) in &self.texts {
            // Calling `.queue()` for all bits of text that can share a `DrawParam`,
            // followed with `::draw_queued()` with said params, is the intended way.
            graphics::queue_text(ctx, text, Point2::new(20.0, 20.0 + height), None);
            height += 20.0 + text.height(ctx) as f32;
        }
        // When drawing via `draw_queued()`, `.offset` in `DrawParam` will be
        // in screen coordinates, and `.color` will be ignored.
        graphics::draw_queued_text(
            ctx,
            DrawParam::default(),
            None,
            graphics::FilterMode::Linear,
        )?;

        // Individual fragments within the `Text` can be replaced;
        // this can be used for inlining animated sentences, words, etc.
        if let Some(text) = self.texts.get_mut("1_demo_text_3") {
            // `.fragments_mut()` returns a mutable slice of contained fragments.
            // Fragments are indexed in order of their addition, starting at 0 (of course).
            text.fragments_mut()[3].color = Some(random_color());
        }

        // Another animation example. Note, this is very inefficient as-is.
        let wobble_string = "WOBBLE";
        let mut wobble = Text::default();
        for ch in wobble_string.chars() {
            wobble.add(
                TextFragment::new(ch).scale(Scale::uniform(10.0 + 6.0 * rand::random::<f32>())),
            );
        }
        let wobble_width = wobble.width(ctx);
        let wobble_height = wobble.height(ctx);
        graphics::queue_text(
            ctx,
            &wobble,
            Point2::new(0.0, 0.0),
            Some(Color::new(0.0, 1.0, 1.0, 1.0)),
        );
        let t = Text::new(format!(
            "width: {}\nheight: {}",
            wobble_width, wobble_height
        ));
        graphics::queue_text(ctx, &t, Point2::new(0.0, 20.0), None);
        graphics::draw_queued_text(
            ctx,
            DrawParam::new()
                .dest(Point2::new(500.0, 300.0))
                .rotation(-0.5),
            None,
            graphics::FilterMode::Linear,
        )?;

        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        graphics::set_screen_coordinates(ctx, graphics::Rect::new(0.0, 0.0, width, height))
            .unwrap();
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

    let (ctx, events_loop) = &mut ContextBuilder::new("text_cached", "ggez")
        .window_setup(WindowSetup::default().title("Cached text example!"))
        .window_mode(
            WindowMode::default()
                .dimensions(640.0, 480.0)
                .resizable(true),
        )
        .add_resource_path(resource_dir)
        .build()?;

    let state = &mut App::new(ctx)?;
    event::run(ctx, events_loop, state)
}
