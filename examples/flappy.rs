extern crate ggez;
extern crate specs;
extern crate rand;
extern crate sdl2;

use rand::Rand;
use sdl2::pixels::Color;

use ggez::conf;
use ggez::{game, Game, State, GameError, Context};
use std::time::Duration;
use std::path::Path;
//use specs::{Join, World};

struct Transform {
    position: (u32, u32),
    //rotation: f32,
}

impl specs::Component for Transform {
    type Storage = specs::VecStorage<Transform>;
}

struct MainState {
    planner: specs::Planner<()>,
    a: i32,
}

impl MainState {
    fn new() -> MainState {
        let mut world = specs::World::new();
        world.register::<Transform>();
        world.create_now()
             .with(Transform {
                 position: (50, 50),
                 //rotation: 0f32,
             })
             .build();
        MainState {
            planner: specs::Planner::new(world, 4),
            a: 0,
        }
    }
}

impl State for MainState {
    fn load(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        println!("load");
        ctx.resources.load_sound("sound", Path::new("./resources/sound.mp3")).unwrap();
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context, _dt: Duration) -> Result<(), GameError> {
        // println!("update");

        self.planner.run1w0r(|t: &mut Transform| {
            t.position.0 += 1;
            t.position.1 += 1;
        });
        self.a = self.a + 1;
        if self.a > 100 {
            self.a = 0;
            try!(game::play_sound(ctx, "sound"));
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        // println!("draw");
        let mut rng = rand::thread_rng();
        ctx.renderer.set_draw_color(Color::rand(&mut rng));
        ctx.renderer.clear();
        ctx.print("roflcopter", 100, 100);
        ctx.renderer.present();

        Ok(())
    }
}

pub fn main() {
    let g = MainState::new();
    let c = conf::Conf::new();
    println!("Default config: {:#?}", c);
    let mut e: Game<MainState> = Game::new(c, g);
    let result = e.run();
    if let Err(e) = result {
        println!("Error encountered: {:?}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
