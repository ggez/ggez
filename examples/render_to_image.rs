extern crate ggez;

use ggez::*;
use ggez::graphics::*;

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

        // now lets render our scene once in the top right and in the bottom
        // right
        let window_size = graphics::get_size(ctx);
        let scale =
            Point2::new(0.5 * window_size.0 as f32 / self.canvas.get_image().width() as f32,
                        0.5 * window_size.1 as f32 / self.canvas.get_image().height() as f32);
        graphics::set_canvas(ctx, None);
        graphics::set_background_color(ctx, Color::new(0.0, 0.0, 0.0, 1.0));
        graphics::clear(ctx);
        graphics::draw_ex(ctx,
                          &self.canvas,
                          DrawParam {
                              dest: Point2::new(0.0, 0.0),
                              scale,
                              ..Default::default()
                          })?;
        graphics::draw_ex(ctx,
                          &self.canvas,
                          DrawParam {
                              dest: Point2::new(400.0, 300.0),
                              scale,
                              ..Default::default()
                          })?;
        graphics::present(ctx);

        Ok(())
    }


    fn resize_event(&mut self, ctx: &mut Context, width: u32, height: u32) {
        let new_rect = graphics::Rect::new(0.0,
                                        0.0,
                                        width as f32,
                                        height as f32);
        graphics::set_screen_coordinates(ctx, new_rect).unwrap();
    }
}

pub fn main() {
    let mut c = conf::Conf::new();
    c.window_setup.resizable = true;
    let ctx = &mut Context::load_from_conf("super_simple", "ggez", c).unwrap();
    let state = &mut MainState::new(ctx).unwrap();
    event::run(ctx, state).unwrap();
}
