//! An example drawing semi-transparent venn diagrams
//! using different blend modes.
//!
//! It also shows why you'd usually want to draw canvases
//! using the `Premultiplied` blend mode
//! (for more explanations on this see https://github.com/ggez/ggez/issues/694#issuecomment-853724926)

use ggez::context::HasMut;
use ggez::event::{self, EventHandler};
use ggez::glam::Vec2;
use ggez::graphics::{self, BlendMode, Color, DrawParam, GraphicsContext};
use ggez::input::keyboard::KeyInput;
use ggez::{Context, GameResult};
use std::env;
use std::path;

struct MainState {
    layer: graphics::ScreenImage,
    layer_blend: BlendMode,
    circle: graphics::Mesh,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let layer = graphics::ScreenImage::new(ctx, None, 1., 1., 1);

        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Vec2::new(0.0, 0.0),
            45.0,
            0.5,
            Color::WHITE,
        )?;

        let s = Self {
            layer,
            layer_blend: BlendMode::PREMULTIPLIED,
            circle,
        };
        Ok(s)
    }

    fn draw_venn(
        &self,
        _gfx: &mut impl HasMut<GraphicsContext>,
        canvas: &mut graphics::Canvas,
        pos: Vec2,
        name: &str,
    ) -> GameResult {
        const TRI_COLORS: [Color; 3] = [
            Color::new(0.8, 0., 0., 0.5),
            Color::new(0., 0.8, 0., 0.5),
            Color::new(0., 0., 0.8, 0.5),
        ];

        const OFFSET: f32 = 24.;
        const REL_POSITIONS: [[f32; 2]; 3] = [
            [-OFFSET, -OFFSET / 2.],
            [OFFSET, -OFFSET / 2.],
            [0., OFFSET],
        ];

        // draw the diagram
        for i in 0..3 {
            canvas.draw(
                &self.circle,
                graphics::DrawParam::new()
                    .dest(pos + Vec2::from(REL_POSITIONS[i]))
                    .color(TRI_COLORS[i]),
            );
        }

        // draw text naming the blend mode
        canvas.set_blend_mode(BlendMode::ALPHA);
        let mut text = graphics::Text::new(name);
        text.set_scale(20.);
        let text_offset = Vec2::new(0., -100.);
        canvas.draw(
            &text,
            graphics::DrawParam::from(pos + text_offset)
                .offset([0.5, 0.0])
                .color(Color::WHITE),
        );

        Ok(())
    }

    fn draw_venn_diagrams(
        &mut self,
        ctx: &mut Context,
        (w, h): (f32, f32),
        canvas: &mut graphics::Canvas,
    ) -> GameResult {
        let y = h / 4.;
        const MODE_COUNT: usize = 8;
        let x_step = w / (MODE_COUNT + 1) as f32;

        // draw with Alpha
        canvas.set_blend_mode(BlendMode::ALPHA);
        self.draw_venn(ctx, canvas, [x_step, y].into(), "Alpha")?;

        // draw with Add
        canvas.set_blend_mode(BlendMode::ADD);
        self.draw_venn(ctx, canvas, [x_step * 2., y].into(), "Add")?;

        // draw with Sub
        canvas.set_blend_mode(BlendMode::SUBTRACT);
        self.draw_venn(ctx, canvas, [x_step * 3., y].into(), "Subtract")?;

        // draw with Multiply
        canvas.set_blend_mode(BlendMode::MULTIPLY);
        self.draw_venn(ctx, canvas, [x_step * 4., y].into(), "Multiply")?;

        // draw with Invert
        canvas.set_blend_mode(BlendMode::INVERT);
        self.draw_venn(ctx, canvas, [x_step * 5., y].into(), "Invert")?;

        // draw with Replace
        canvas.set_blend_mode(BlendMode::REPLACE);
        self.draw_venn(ctx, canvas, [x_step * 6., y].into(), "Replace")?;

        // draw with Darken
        canvas.set_blend_mode(BlendMode::DARKEN);
        self.draw_venn(ctx, canvas, [x_step * 7., y].into(), "Darken")?;

        // draw with Lighten
        canvas.set_blend_mode(BlendMode::LIGHTEN);
        self.draw_venn(ctx, canvas, [x_step * 8., y].into(), "Lighten")?;

        Ok(())
    }
}

impl EventHandler for MainState {
    fn update(&mut self, _: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let (w, h) = ctx.gfx.drawable_size();

        // draw everything onto self.layer
        let layer = self.layer.image(ctx);
        let mut canvas =
            graphics::Canvas::from_image(ctx, layer.clone(), Color::new(0., 0., 0., 0.));
        self.draw_venn_diagrams(ctx, (w, h), &mut canvas)?;
        canvas.finish(ctx)?;

        // now start drawing to the screen
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::new(0.3, 0.3, 0.3, 1.0));

        // draw everything directly onto the screen once
        self.draw_venn_diagrams(ctx, (w, h), &mut canvas)?;

        // draw layer onto the screen
        canvas.set_blend_mode(self.layer_blend);
        canvas.draw(
            &layer,
            DrawParam::default().dest(mint::Point2 { x: 0., y: h / 2. }),
        );

        // draw text pointing out which is which
        let y = h / 2.;

        canvas.draw(
            graphics::Text::new("drawn directly:").set_scale(20.),
            graphics::DrawParam::from([8., 4.]).color(Color::WHITE),
        );
        canvas.draw(
            graphics::Text::new("drawn onto a (transparent black) canvas:").set_scale(20.),
            graphics::DrawParam::from([8., 4. + y]).color(Color::WHITE),
        );

        canvas.finish(ctx)?;

        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, _input: KeyInput, repeat: bool) -> GameResult {
        if !repeat {
            if self.layer_blend == BlendMode::ALPHA {
                self.layer_blend = BlendMode::PREMULTIPLIED;
                println!("Drawing canvas with premultiplied alpha mode");
            } else {
                self.layer_blend = BlendMode::ALPHA;
                println!("Drawing canvas with default alpha mode");
            }
        }
        Ok(())
    }
}

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("blend_modes", "ggez")
        .window_mode(ggez::conf::WindowMode::default().dimensions(1400., 600.))
        .window_setup(
            ggez::conf::WindowSetup::default()
                .title("blend modes -- Press a button to change the canvas blend mode!"),
        )
        .add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
