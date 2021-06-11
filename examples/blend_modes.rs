//! An example drawing semi-transparent venn diagrams
//! using different blend modes.
//!
//! It also shows why you'd usually want to draw canvases
//! using the `Premultiplied` blend mode
//! (for more explanations on this see https://github.com/ggez/ggez/issues/694#issuecomment-853724926)

use ggez::event::{self, EventHandler};
use ggez::graphics::{self, BlendMode, Color, DrawParam, Drawable};
use ggez::{Context, GameResult};
use glam::Vec2;
use std::env;
use std::path;

struct MainState {
    circle: graphics::Mesh,
    canvas: graphics::Canvas,
    font: graphics::Font,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Vec2::new(0.0, 0.0),
            45.0,
            0.5,
            Color::WHITE,
        )?;

        let mut canvas = graphics::Canvas::with_window_size(ctx)?;
        canvas.set_blend_mode(Some(BlendMode::Alpha));

        let font = graphics::Font::new(ctx, "/LiberationMono-Regular.ttf")?;

        let s = Self {
            circle,
            canvas,
            font,
        };
        Ok(s)
    }

    fn draw_venn(&self, ctx: &mut Context, pos: Vec2, name: &str) -> GameResult<()> {
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
            self.circle.draw(
                ctx,
                DrawParam::default()
                    .dest(pos + Vec2::from(REL_POSITIONS[i]))
                    .color(TRI_COLORS[i]),
            )?;
        }

        // draw text naming the blend mode
        let text = graphics::Text::new((name, self.font, 20.0));
        let text_offset = Vec2::new(0., -100.);
        graphics::draw(
            ctx,
            &text,
            graphics::DrawParam::new()
                .dest(pos + text_offset)
                .color(Color::WHITE)
                .offset(Vec2::new(0.5, 0.5)),
        )?;
        Ok(())
    }

    fn draw_venn_diagrams(&mut self, ctx: &mut Context) -> GameResult<()> {
        let (w, h) = graphics::drawable_size(ctx);
        let y = h / 4.;
        const MODE_COUNT: usize = 8;
        let x_step = w / (MODE_COUNT + 1) as f32;

        // draw with Alpha
        self.circle.set_blend_mode(Some(BlendMode::Alpha));
        self.draw_venn(ctx, [x_step, y].into(), "Alpha")?;

        // draw with Add
        self.circle.set_blend_mode(Some(BlendMode::Add));
        self.draw_venn(ctx, [x_step * 2., y].into(), "Add")?;

        // draw with Sub
        self.circle.set_blend_mode(Some(BlendMode::Subtract));
        self.draw_venn(ctx, [x_step * 3., y].into(), "Subtract")?;

        // draw with Multiply
        self.circle.set_blend_mode(Some(BlendMode::Multiply));
        self.draw_venn(ctx, [x_step * 4., y].into(), "Multiply")?;

        // draw with Invert
        self.circle.set_blend_mode(Some(BlendMode::Invert));
        self.draw_venn(ctx, [x_step * 5., y].into(), "Invert")?;

        // draw with Replace
        self.circle.set_blend_mode(Some(BlendMode::Replace));
        self.draw_venn(ctx, [x_step * 6., y].into(), "Replace")?;

        // draw with Darken
        self.circle.set_blend_mode(Some(BlendMode::Darken));
        self.draw_venn(ctx, [x_step * 7., y].into(), "Darken")?;

        // draw with Lighten
        self.circle.set_blend_mode(Some(BlendMode::Lighten));
        self.draw_venn(ctx, [x_step * 8., y].into(), "Lighten")?;

        Ok(())
    }
}

impl EventHandler for MainState {
    fn update(&mut self, _: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::new(0.3, 0.3, 0.3, 1.0));

        // draw everything directly onto the screen once
        self.draw_venn_diagrams(ctx)?;

        // also draw everything onto the canvas
        graphics::set_canvas(ctx, Some(&self.canvas));
        graphics::clear(ctx, Color::new(0., 0., 0., 0.));
        self.draw_venn_diagrams(ctx)?;

        // draw the canvas onto the screen
        graphics::set_canvas(ctx, None);
        let (_, height) = graphics::drawable_size(ctx);
        self.canvas.draw(
            ctx,
            DrawParam::default().dest(mint::Point2 {
                x: 0.,
                y: height / 2.,
            }),
        )?;

        // draw text pointing out which is which
        let (_w, h) = graphics::drawable_size(ctx);
        let y = h / 2.;

        let text = graphics::Text::new(("drawn directly:", self.font, 20.0));
        graphics::draw(
            ctx,
            &text,
            graphics::DrawParam::new()
                .dest(Vec2::new(8., 4.))
                .color(Color::WHITE),
        )?;
        let text =
            graphics::Text::new(("drawn onto a (transparent black) canvas:", self.font, 20.0));
        graphics::draw(
            ctx,
            &text,
            graphics::DrawParam::new()
                .dest(Vec2::new(8., 4. + y))
                .color(Color::WHITE),
        )?;

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        _keycode: ggez::event::KeyCode,
        _keymod: ggez::event::KeyMods,
        repeat: bool,
    ) {
        if !repeat {
            if let Some(BlendMode::Alpha) = self.canvas.blend_mode() {
                self.canvas.set_blend_mode(Some(BlendMode::Premultiplied));
                println!("Drawing canvas with premultiplied alpha mode");
            } else {
                self.canvas.set_blend_mode(Some(BlendMode::Alpha));
                println!("Drawing canvas with default alpha mode");
            }
        }
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
