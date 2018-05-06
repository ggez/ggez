extern crate ggez;
extern crate rand;

use ggez::conf;
use ggez::event;
use ggez::graphics::{self, DrawMode};
use ggez::{Context, ContextBuilder, GameResult};

/**
 * The board is assumed to be square, with dimensions set in the main function.
 *
 * The number of columns and rows is determined by the width of the window
 * divided by CELL_SIZE
 */

const CELL_SIZE: f32 = 10.0;

struct MainState {
    curr: Box<Vec<Vec<bool>>>,
    prev: Box<Vec<Vec<bool>>>,
    limit: usize,
}

fn wrap(idx: i32, size: i32, amt: i32) -> usize {
    let mut res = idx + amt;
    if res < 0 {
        res = size + res;
    } else {
        res = res % size;
    }
    res as usize
}

fn is_alive(i: usize, j: usize, board: &Vec<Vec<bool>>) -> bool {
    let mut neighbors_alive = 0;
    for n in -1..1 + 1 {
        for m in -1..1 + 1 {
            if (n != 0 || m != 0)
                && board[wrap(i as i32, board.len() as i32, n as i32)]
                    [wrap(j as i32, board.len() as i32, m as i32)]
            {
                neighbors_alive += 1;
            }
        }
    }
    neighbors_alive == 3 || neighbors_alive == 2 && board[i][j]
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        // columns/rows count set here
        let limit = ctx.conf.window_mode.width as usize / CELL_SIZE as usize;
        let mut board: Vec<_> = Vec::new();
        for i in 0..limit {
            board.push(vec![]);
            for _ in 0..limit {
                board[i].push(rand::random::<f64>() < 0.15);
            }
        }

        let s = MainState {
            curr: Box::new(board.clone()),
            prev: Box::new(board),
            limit,
        };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        for i in 0..self.limit {
            for j in 0..self.limit {
                self.prev[i][j] = is_alive(i, j, &self.curr);
            }
        }
        std::mem::swap(&mut self.curr, &mut self.prev);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        for i in 0..self.limit {
            for j in 0..self.limit {
                if self.curr[i][j] {
                    graphics::rectangle(
                        ctx,
                        DrawMode::Fill,
                        graphics::Rect {
                            x: i as f32 * CELL_SIZE,
                            y: j as f32 * CELL_SIZE,
                            w: CELL_SIZE,
                            h: CELL_SIZE,
                        },
                    )?;
                }
            }
        }
        graphics::present(ctx);
        Ok(())
    }
}

pub fn main() {
    let cb = ContextBuilder::new("game_of_life", "ggez")
        .window_setup(conf::WindowSetup::default().title("Game of Life"))
        .window_mode(conf::WindowMode::default().dimensions(800, 800));
    let ctx = &mut cb.build().unwrap();
    let state = &mut MainState::new(ctx).unwrap();
    event::run(ctx, state).unwrap();
}
