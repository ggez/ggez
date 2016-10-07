//! An Asteroids-ish example game to show off ggez.
//! The idea is that this game is simple but still
//! non-trivial enough to be interesting.

extern crate ggez;
extern crate rand;
extern crate sdl2;

use std::path;
use sdl2::pixels::Color;

use ggez::audio;
use ggez::conf;
use ggez::game::{Game, GameState};
use ggez::{GameResult, Context};
use ggez::graphics;
use ggez::timer;
use std::time::Duration;

struct Coordinate {
    facing: f32,
    location: graphics::Point,
}

struct Shot {
    coord: Coordinate,
}

struct Player {
    coord: Coordinate,
}

struct Rock {
    coord: Coordinate,
}

struct Assets {
    player_image: graphics::Image,
    shot_image: graphics::Image,
    rock_image: graphics::Image,
}

impl Assets {
    fn new(ctx: &mut Context) -> GameResult<Assets> {
        let player_image_path = path::Path::new("player.png");
        let player_image = try!(graphics::Image::new(ctx, player_image_path));
        let shot_image_path = path::Path::new("shot.png");
        let shot_image = try!(graphics::Image::new(ctx, shot_image_path));
        let rock_image_path = path::Path::new("rock.png");
        let rock_image = try!(graphics::Image::new(ctx, rock_image_path));
        Ok(Assets {
            player_image: player_image,
            shot_image: shot_image,
            rock_image: rock_image,
        })
    }
}


struct MainState {
    //player: Player,
    shots: Vec<Shot>,
    rocks: Vec<Rock>,
    score: u32,
    assets: Assets,
}

//impl MainState {
//}

impl GameState for MainState {
    fn load(ctx: &mut Context, _conf: &conf::Conf) -> GameResult<MainState> {
        ctx.print_sound_stats();
        ctx.print_resource_stats();
        graphics::set_background_color(ctx, Color::RGB(0, 0, 0));

        let assets = try!(Assets::new(ctx));
        
        let s = MainState {
            shots: Vec::new(),
            rocks: Vec::new(),
            score: 0,
            assets: assets,
        };

        Ok(s)
    }

    fn update(&mut self, _ctx: &mut Context, _dt: Duration) -> GameResult<()> {

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        timer::sleep_until_next_frame(ctx, 60);
        // ctx.quit() is broken :-(
        //ctx.quit();
        Ok(())
    }
}

pub fn main() {
    let c = conf::Conf::new("Astroblasto!");
    let game: GameResult<Game<MainState>> = Game::new(c);
    match game {
        Err(e) => {
            println!("Could not load game!");
            println!("Error: {:?}", e);
        }
        Ok(mut game) => {
            let result = game.run();
            if let Err(e) = result {
                println!("Error encountered running game: {:?}", e);
            } else {
                println!("Game exited cleanly.");
            }
        }
    }
}

