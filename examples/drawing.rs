extern crate ggez;
use ggez::conf;
use ggez::event;
use ggez::{GameResult, Context};
use ggez::graphics;
use ggez::graphics::{DrawMode, Point};
use std::time::Duration;

struct MainState {
    image1: graphics::Image,
    image2: graphics::Image,
    zoomlevel: f32,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {

        let image1 = graphics::Image::new(ctx, "/dragon1.png")?;
        let image2 = graphics::Image::new(ctx, "/player.png")?;
        let s = MainState {
            image1: image1,
            image2: image2,
            zoomlevel: 1.0,
        };

        Ok(s)
    }
}


impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context, _dt: Duration) -> GameResult<()> {
        self.zoomlevel += 0.01;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        // let src = graphics::Rect::new(0.25, 0.25, 0.5, 0.5);
        // let src = graphics::Rect::one();
        let dst = graphics::Point::new(200.0, 200.0);
        graphics::draw(ctx, &mut self.image1, dst, 0.0)?;
        let dst = graphics::Point::new(100.0, 100.0);
        let scale = graphics::Point::new(4.0, 4.0);
        // let shear = graphics::Point::new(self.zoomlevel, self.zoomlevel);
        // graphics::set_color(ctx, graphics::Color::new(1.0, 1.0, 1.0, 1.0));
        graphics::draw_ex(ctx,
                          &mut self.image2,
                          graphics::DrawParam {
                              // src: src,
                              dest: dst,
                              rotation: self.zoomlevel,
                              offset: Point::new(-16.0, 0.0),
                              scale: scale,
                              // shear: shear,
                              ..Default::default()
                          })?;

        let rect = graphics::Rect::new(450.0, 450.0, 50.0, 50.0);
        graphics::rectangle(ctx, graphics::DrawMode::Fill, rect)?;

        graphics::set_color(ctx, graphics::Color::new(1.0, 0.0, 0.0, 1.0))?;
        let rect = graphics::Rect::new(450.0, 450.0, 50.0, 50.0);
        graphics::rectangle(ctx, graphics::DrawMode::Line, rect)?;
        graphics::set_color(ctx, graphics::WHITE)?;

        graphics::set_line_width(ctx, 4.0);
        graphics::line(ctx,
                       &[Point {
                             x: 200.0,
                             y: 200.0,
                         },
                         Point {
                             x: 400.0,
                             y: 200.0,
                         },
                         Point {
                             x: 400.0,
                             y: 400.0,
                         },
                         Point {
                             x: 200.0,
                             y: 400.0,
                         },
                         Point {
                             x: 200.0,
                             y: 200.0,
                         }])?;

        graphics::ellipse(ctx,
                          DrawMode::Fill,
                          Point {
                              x: 600.0,
                              y: 200.0,
                          },
                          50.0,
                          120.0,
                          32)?;

        graphics::circle(ctx,
                         DrawMode::Fill,
                         Point {
                             x: 600.0,
                             y: 380.0,
                         },
                         40.0,
                         32)?;

        graphics::present(ctx);
        Ok(())
    }
}

pub fn main() {
    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("drawing", "ggez", c).unwrap();
    println!("{}", graphics::get_renderer_info(ctx).unwrap());
    let state = &mut MainState::new(ctx).unwrap();
    if let Err(e) = event::run(ctx, state) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
