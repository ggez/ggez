//! A small snake game done after watching
//! <https://www.youtube.com/watch?v=HCwMb0KslX8>
//! to showcase ggez and how it relates/differs from piston.
//!
//! Author: @termhn
//! Original repo: https://github.com/termhn/ggez_snake

extern crate ggez;
extern crate rand;

use ggez::{event, graphics, GameResult, Context};
use ggez::event::Keycode;

use std::collections::LinkedList;
use std::time::{Instant, Duration};

use rand::{Rng};

const GRID_SIZE: (i16, i16) = (30, 20);
const GRID_CELL_SIZE: (i16, i16) = (32, 32);

const SCREEN_SIZE: (u32, u32) = (GRID_SIZE.0 as u32 * GRID_CELL_SIZE.0 as u32,
                                 GRID_SIZE.1 as u32 * GRID_CELL_SIZE.1 as u32);

const UPDATES_PER_SECOND: f32 = 8.0;
const MILLIS_PER_UPDATE: u64 = (1.0 / UPDATES_PER_SECOND * 1000.0) as u64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct GridPosition {
    x: i16,
    y: i16,
}

/// A trait that provides a modulus function that works for negative values
/// rather than just the standard remainder op (%) which does not.
trait ModuloSigned {
    fn modulo(&self, n: Self) -> Self;
}

impl<T> ModuloSigned for T
    where T: std::ops::Add<Output = T> + std::ops::Rem<Output = T> + Clone
{
    fn modulo(&self, n: T) -> T {
        (self.clone() % n.clone() + n.clone()) % n.clone()
    }
}

impl GridPosition {
    pub fn new(x: i16, y: i16) -> Self {
        GridPosition { x, y }
    }

    pub fn random(max_x: i16, max_y: i16) -> Self {
        let mut rng = rand::thread_rng();
        (rng.gen_range::<i16>(0, max_x), 
         rng.gen_range::<i16>(0, max_y)).into()
    }

    pub fn new_from_move(pos: GridPosition, dir: Direction) -> Self {
        match dir {
            Direction::Up => GridPosition::new(pos.x, (pos.y - 1).modulo(GRID_SIZE.1)),
            Direction::Down => GridPosition::new(pos.x, (pos.y + 1).modulo(GRID_SIZE.1)),
            Direction::Left => GridPosition::new((pos.x - 1).modulo(GRID_SIZE.0), pos.y),
            Direction::Right => GridPosition::new((pos.x + 1).modulo(GRID_SIZE.0), pos.y),
        }
    }
}

impl From<GridPosition> for graphics::Rect {
    fn from(pos: GridPosition) -> Self {
        graphics::Rect::new_i32(pos.x as i32 * GRID_CELL_SIZE.0 as i32, pos.y as i32 * GRID_CELL_SIZE.1 as i32,
                                GRID_CELL_SIZE.0 as i32, GRID_CELL_SIZE.1 as i32)
    }
}

impl From<(i16, i16)> for GridPosition {
    fn from(pos: (i16, i16)) -> Self {
        GridPosition { x: pos.0, y: pos.1 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn inverse(&self) -> Self {
        match *self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    pub fn from_keycode(key: Keycode) -> Option<Direction> {
        match key {
            Keycode::Up => Some(Direction::Up),
            Keycode::Down => Some(Direction::Down),
            Keycode::Left => Some(Direction::Left),
            Keycode::Right => Some(Direction::Right),
            _ => None
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Segment {
    pos: GridPosition,
}

impl Segment {
    pub fn new(pos: GridPosition) -> Self {
        Segment { pos }
    }
}

#[derive(Clone, Copy, Debug)]
enum Ate {
    Itself,
    Food,
}

struct Snake {
    head: Segment,
    dir: Direction,
    body: LinkedList<Segment>,
    ate: Option<Ate>,
    last_update_dir: Direction,
}

impl Snake {
    pub fn new(pos: GridPosition) -> Self {
        let mut body = LinkedList::new();
        body.push_back(Segment::new((pos.x - 1, pos.y).into()));
        Snake {
            head: Segment::new(pos),
            dir: Direction::Right,
            last_update_dir: Direction::Right,
            body: body,
            ate: None,
        }
    }

    fn eats(&self, food: &Food) -> bool {
        if self.head.pos == food.pos {
            true
        } else {
            false
        }
    }

    fn eats_self(&self) -> bool {
        for seg in self.body.iter() {
            if self.head.pos == seg.pos {
                return true;
            }
        }
        false
    }

    fn update(&mut self, food: &Food) {
        let new_head_pos = GridPosition::new_from_move(self.head.pos, self.dir);
        let new_head = Segment::new(new_head_pos);
        self.body.push_front(self.head);
        self.head = new_head;
        if self.eats_self() {
            self.ate = Some(Ate::Itself);
        } else if self.eats(food) {
            self.ate = Some(Ate::Food);
        } else {
            self.ate = None
        }
        if let None = self.ate {
            self.body.pop_back();
        }
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        for seg in self.body.iter() {
            graphics::set_color(ctx, [1.0, 0.5, 0.0, 1.0].into())?;
            graphics::rectangle(ctx, graphics::DrawMode::Fill, seg.pos.into())?;
        }
        graphics::set_color(ctx, [1.0, 0.0, 0.0, 1.0].into())?;
        graphics::rectangle(ctx, graphics::DrawMode::Fill, self.head.pos.into())?;
        Ok(())
    }
}

struct Food {
    pos: GridPosition
}

impl Food {
    pub fn new(pos: GridPosition) -> Self {
        Food { pos }
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, [0.0, 0.0, 1.0, 1.0].into())?;
        graphics::rectangle(ctx, graphics::DrawMode::Fill, self.pos.into())
    }
}

struct GameState {
    snake: Snake,
    food: Food,
    gameover: bool,
    last_update: Instant,
}

impl GameState {
    pub fn new() -> Self {
        let snake_pos = (GRID_SIZE.0 / 4, GRID_SIZE.1 / 2).into();
        let food_pos = GridPosition::random(GRID_SIZE.0, GRID_SIZE.1);

        GameState {
            snake: Snake::new(snake_pos),
            food: Food::new(food_pos),
            gameover: false,
            last_update: Instant::now(),
        }
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        if Instant::now() - self.last_update >= Duration::from_millis(MILLIS_PER_UPDATE) {
            if !self.gameover {
                self.snake.update(&self.food);
                if let Some(ate) = self.snake.ate {
                    match ate {
                        Ate::Food => {
                            let new_food_pos = GridPosition::random(GRID_SIZE.0, GRID_SIZE.1);
                            self.food.pos = new_food_pos;
                        },
                        Ate::Itself => {
                            self.gameover = true;
                        }
                    }
                }
            }
            self.last_update = Instant::now();
            self.snake.last_update_dir = self.snake.dir;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        self.snake.draw(ctx)?;
        self.food.draw(ctx)?;
        graphics::present(ctx);
        ggez::timer::yield_now();
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: Keycode, _keymod: event::Mod, _repeat: bool) {
        if let Some(dir) = Direction::from_keycode(keycode) {
            if dir.inverse() != self.snake.last_update_dir {
                self.snake.dir = dir;
            }
        }
    }
}

fn main() {
    let ctx = &mut ggez::ContextBuilder::new("snake", "Gray Olson")
        .window_setup(ggez::conf::WindowSetup::default().title("Snake!"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build().expect("Failed to build ggez context");

    graphics::set_background_color(ctx, [0.0, 1.0, 0.0, 1.0].into());

    let state = &mut GameState::new();

    match event::run(ctx, state) {
        Err(e) => println!("Error encountered running game: {}", e),
        Ok(_) => println!("Game exited cleanly!")
    }
}
