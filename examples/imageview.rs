extern crate ggez;
extern crate specs;
extern crate rand;
extern crate sdl2;

use std::path;
use sdl2::pixels::Color;

use ggez::audio;
use ggez::conf;
use ggez::{Game, State, GameResult, Context};
use ggez::graphics;
use ggez::graphics::Drawable;
use std::time::Duration;

struct MainState {
    a: i32,
    direction: i32,
    image: Option<graphics::Image>,
    font: Option<graphics::Font>,
    text: Option<graphics::Text>,
    sound: Option<audio::Sound>,
}

impl MainState {
    fn new() -> MainState {
        MainState {
            a: 0,
            direction: 1,
            image: None,
            font: None,
            text: None,
            sound: None,

        }
    }
}

impl State for MainState {
    fn load(&mut self, ctx: &mut Context) -> GameResult<()> {
        ctx.print_sound_stats();
        ctx.print_resource_stats();

        let imagepath = path::Path::new("dragon1.png");
        let image = graphics::Image::new(ctx, imagepath).unwrap();

        let fontpath = path::Path::new("DejaVuSerif.ttf");
        let soundpath = path::Path::new("sound.ogg");
        let font = graphics::Font::new(ctx, fontpath, 48).unwrap();
        let text = graphics::Text::new(ctx, "Hello world!", &font).unwrap();
        let sound = audio::Sound::new(ctx, soundpath).unwrap();
        self.image = Some(image);
        self.font = Some(font);
        self.text = Some(text);
        self.sound = Some(sound);


        let sound = self.sound.as_ref().unwrap();
        let _ = sound.play();

        Ok(())
    }

    fn update(&mut self, _ctx: &mut Context, _dt: Duration) -> GameResult<()> {
        // println!("update");

        self.a = self.a + self.direction;
        if self.a > 250 || self.a <= 0 {
            self.direction *= -1;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let c = self.a as u8;
        ctx.renderer.set_draw_color(Color::RGB(c, c, c));
        ctx.renderer.clear();

        let img = self.image.as_ref().unwrap();
        img.draw(ctx, None, None);
        let text = self.text.as_ref().unwrap();
        text.draw(ctx, None, None);
        ctx.renderer.present();


        Ok(())
    }
}

// Creating a gamestate depends on having an SDL context to load resources.
// Creating a context depends on loading a config file.
// Loading a config file depends on having FS (or we can just fake our way around it
// by creating an FS and then throwing it away; the costs are not huge.)
pub fn main() {
    let g = MainState::new();
    let c = conf::Conf::new("flappy");
    println!("Starting with default config: {:#?}", c);
    let mut e: Game<MainState> = Game::new(g, c).unwrap();
    let result = e.run();
    if let Err(e) = result {
        println!("Error encountered: {:?}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
