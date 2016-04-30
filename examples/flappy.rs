extern crate ggez;
extern crate specs;

use ggez::{Game, State, GameError};
use std::time::Duration;
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
        MainState { planner: specs::Planner::new(world, 4) }
    }
}

impl State for MainState {
    fn load(&mut self) -> Result<(), GameError> {
        println!("load");
        Ok(())
    }

    fn update(&mut self, dt: Duration) -> Result<(), GameError> {
        println!("update");
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
