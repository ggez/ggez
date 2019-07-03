//! An example of how to draw to `Image`'s using the `Canvas` type.

use cgmath;
use ggez;

use ggez::event;
use ggez::graphics::{self, Color, DrawParam};
use ggez::{Context, GameResult};

type Point2 = cgmath::Point2<f32>;
type Vector2 = cgmath::Vector2<f32>;

struct MainState {
    canvas: graphics::Canvas,
    text: graphics::Text,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let canvas = graphics::Canvas::with_window_size(ctx)?;
        let font = graphics::Font::default();
        let text = graphics::Text::new(("Hello world!", font, 24.0));
        Ok(MainState { canvas, text })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // first lets render to our canvas
        graphics::set_canvas(ctx, Some(&self.canvas));
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
        graphics::draw(
            ctx,
            &self.text,
            (Point2::new(400.0, 300.0), graphics::WHITE),
        )?;

        // now lets render our scene once in the top left and in the bottom
        // right
        let window_size = graphics::size(ctx);
        let scale = Vector2::new(
            0.5 * window_size.0 as f32 / self.canvas.image().width() as f32,
            0.5 * window_size.1 as f32 / self.canvas.image().height() as f32,
        );
        // let scale = Vector2::new(1.0, 1.0);
        graphics::set_canvas(ctx, None);
        graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 1.0));
        graphics::draw(
            ctx,
            &self.canvas,
            DrawParam::default()
                .dest(Point2::new(0.0, 0.0))
                .scale(scale),
        )?;
        graphics::draw(
            ctx,
            &self.canvas,
            DrawParam::default()
                .dest(Point2::new(400.0, 300.0))
                .scale(scale),
        )?;
        graphics::present(ctx)?;

        Ok(())
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        let new_rect = graphics::Rect::new(0.0, 0.0, width, height);
        graphics::set_screen_coordinates(ctx, new_rect).unwrap();
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("render_to_image", "ggez");
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
