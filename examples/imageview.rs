extern crate ggez;
extern crate rand;
extern crate sdl2;


use ggez::audio;
use ggez::conf;
use ggez::game::{Game, GameState};
use ggez::{GameResult, Context};
use ggez::graphics;
use ggez::graphics::Color;
use ggez::timer;
use std::time::Duration;

struct MainState {
    a: i32,
    direction: i32,
    image: graphics::Image,
    text: graphics::Text,
    bmptext: graphics::Text,
    // Not actually dead, see BUGGO below
    #[allow(dead_code)]
    sound: audio::Sound,
}

impl MainState {
    fn draw_crazy_lines(&mut self, ctx: &mut Context) -> GameResult<()> {
        let num_lines = 100;
        let mut colors = Vec::new();
        for _ in 0..num_lines {
            let r: u8 = rand::random();
            let g: u8 = rand::random();
            let b: u8 = rand::random();
            colors.push(Color::RGB(r, g, b));
        }

        let mut last_point = graphics::Point::new(400, 300);
        for color in colors {
            let x = rand::random::<i32>() % 50;
            let y = rand::random::<i32>() % 50;
            let point = graphics::Point::new(last_point.x() + x, last_point.y() + y);
            graphics::set_color(ctx, color);
            try!(graphics::line(ctx, last_point, point));
            last_point = point;
        }

        Ok(())
    }
}

impl GameState for MainState {
    fn load(ctx: &mut Context) -> GameResult<MainState> {
        ctx.print_sound_stats();
        ctx.print_resource_stats();

        let image = graphics::Image::new(ctx, "dragon1.png").unwrap();

        let font = graphics::Font::new(ctx, "DejaVuSerif.ttf", 48).unwrap();
        let text = graphics::Text::new(ctx, "Hello world!", &font).unwrap();
        let bmpfont = graphics::Font::new_bitmap(ctx, "arial.png", "ABCDEFGHIJKLMNOPQRSTUVWXYZ")
            .unwrap();
        let bmptext = graphics::Text::new(ctx, "ZYXWVYTSRQPONMLKJIHGFEDCBA", &bmpfont).unwrap();
        let sound = audio::Sound::new(ctx, "sound.ogg").unwrap();

        let _ = sound.play();

        let s = MainState {
            a: 0,
            direction: 1,
            image: image,
            text: text,
            bmptext: bmptext,
            // BUGGO: We never use sound again,
            // but we have to hang on to it, Or Else!
            // The optimizer will decide we don't need it
            // since play() has "no side effects" and free it.
            // Or something.
            sound: sound,
        };



        Ok(s)
    }

    fn update(&mut self, _ctx: &mut Context, _dt: Duration) -> GameResult<()> {
        self.a += self.direction;
        if self.a > 250 || self.a <= 0 {
            self.direction *= -1;

            println!("Delta frame time: {:?} ", _dt);
            println!("Average FPS: {}", timer::get_fps(_ctx));
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let c = self.a as u8;
        ctx.renderer.set_draw_color(Color::RGB(c, c, c));
        ctx.renderer.clear();

        try!(graphics::draw(ctx, &mut self.image, None, None));
        try!(graphics::draw(ctx, &mut self.text, None, None));
        let destrect = graphics::Rect::new(100, 50, 403, 50);
        try!(graphics::draw(ctx, &mut self.bmptext, None, Some(destrect)));

        try!(self.draw_crazy_lines(ctx));
        ctx.renderer.present();

        timer::sleep_until_next_frame(ctx, 60);
        // ctx.quit() is broken :-(
        // ctx.quit();
        Ok(())
    }
}

// Creating a gamestate depends on having an SDL context to load resources.
// Creating a context depends on loading a config file.
// Loading a config file depends on having FS (or we can just fake our way around it
// by creating an FS and then throwing it away; the costs are not huge.)
pub fn main() {
    let c = conf::Conf::new();
    println!("Starting with default config: {:#?}", c);
    let mut e: Game<MainState> = Game::new("imageview", c).unwrap();
    let result = e.run();
    if let Err(e) = result {
        println!("Error encountered: {:?}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
