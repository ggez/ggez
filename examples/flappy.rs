extern crate ggez;
extern crate specs;

use ggez::{Game, State, GameError, Context};
use std::time::Duration;
use std::path::Path;
use specs::{Join, World};

struct Transform {
    position: (u32, u32),
    rotation: f32,
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
                 rotation: 0f32,
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
        ctx.resources.load_sound("sound", Path::new("./resources/sound.mp3"));
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context, dt: Duration) -> Result<(), GameError> {
        println!("update");
        self.planner.run1w0r(|t: &mut Transform| {
            t.position.0 += 1;
            t.position.1 += 1;
        });
        self.a = self.a + 1;
        if self.a > 100 {
            self.a = 0;
            // let _ : () = Game::play_sound(ctx, "sound");
        }
        Ok(())
    }

    fn draw(&mut self) -> Result<(), GameError> {
        println!("draw");
        Ok(())
    }
}

pub fn main() {
    let mut g = MainState::new();
    let mut e: Game<MainState> = Game::new(g);
    e.run();
}
