extern crate ggez;
extern crate specs;
extern crate rand;
extern crate sdl2;

use std::path;
use rand::Rand;
use sdl2::pixels::Color;

use ggez::audio;
use ggez::conf;
use ggez::{game, Game, State, GameError, GameResult, Context};
use ggez::graphics;
use ggez::graphics::Drawable;
use std::time::Duration;
use std::path::Path;

struct MainState {
    a: i32,
    buffer: Vec<u8>,
    image: Option<graphics::Image>,
    font: Option<graphics::Font>,
    text: Option<graphics::Text>,
    sound: Option<audio::Sound>,
}

impl MainState {
    fn new() -> MainState {
        MainState {
            a: 0,
            image: None,
            font: None,
            text: None,
            sound: None,

            buffer: Vec::new(),
        }
    }
}

impl State for MainState {
    fn load(&mut self, ctx: &mut Context) -> GameResult<()> {
        println!("load");

        let imagepath = path::Path::new("dragon1.png");
        let image = graphics::Image::new(ctx, imagepath).unwrap();

        let fontpath = path::Path::new("DejaVuSerif.ttf");
        let soundpath = path::Path::new("sound.ogg");
        let font = graphics::Font::new(ctx, fontpath, 24).unwrap();
        let text = graphics::Text::new(ctx, "Hello world!", &font).unwrap();
        let sound = audio::Sound::new(ctx, soundpath).unwrap();
        self.image = Some(image);
        self.font = Some(font);
        self.text = Some(text);
        self.sound = Some(sound);


        let sound = self.sound.as_ref().unwrap();
        sound.play();

        Ok(())
    }

    fn update(&mut self, ctx: &mut Context, _dt: Duration) -> GameResult<()> {
        // println!("update");

        self.a = self.a + 1;
        if self.a > 100 {
            self.a = 0;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // println!("draw");
        let mut rng = rand::thread_rng();
        ctx.renderer.set_draw_color(Color::rand(&mut rng));
        ctx.renderer.clear();

        //let img: &ggez::graphics::Image = self.image.as_ref().unwrap();
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
    println!("Default config: {:#?}", c);
    let mut e: Game<MainState> = Game::new(c, g);
    let result = e.run();
    if let Err(e) = result {
        println!("Error encountered: {:?}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
