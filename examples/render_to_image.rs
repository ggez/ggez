extern crate ggez;

use ggez::*;
use ggez::graphics::{Canvas, Color, DrawMode, DrawParam, Point2};

struct MainState {
    canvas: Canvas,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let canvas = Canvas::with_window_size(ctx)?;
        Ok(MainState { canvas })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // first lets render to our canvas
        graphics::set_canvas(ctx, Some(&self.canvas));
        graphics::set_background_color(ctx, Color::new(0.1, 0.2, 0.3, 1.0));
        graphics::clear(ctx);
        graphics::circle(ctx, DrawMode::Fill, Point2::new(400.0, 300.0), 100.0, 2.0)?;
        graphics::present(ctx);

        // now lets render our scene once in the top right and in the bottom
        // right
        graphics::set_canvas(ctx, None);
        graphics::set_background_color(ctx, Color::new(0.0, 0.0, 0.0, 1.0));
        graphics::clear(ctx);
        graphics::draw_ex(
            ctx,
            self.canvas.get_image(),
            DrawParam {
                dest: Point2::new(200.0, 150.0),
                scale: Point2::new(0.5, 0.5),
                ..Default::default()
            },
        )?;
        graphics::draw_ex(
            ctx,
            self.canvas.get_image(),
            DrawParam {
                dest: Point2::new(600.0, 450.0),
                scale: Point2::new(0.5, 0.5),
                ..Default::default()
            },
        )?;
        graphics::present(ctx);

        Ok(())
    }
}

pub fn main() {
    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("super_simple", "ggez", c).unwrap();
    let state = &mut MainState::new(ctx).unwrap();
    event::run(ctx, state).unwrap();
}
