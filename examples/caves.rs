//! Cave generator using Cellular Automata techniques. This
//! generator is based on tutorials for cave generation and
//! bitmask autotiling that can be found at the following links:
//! * [Cellular Automata Cave Generation](http://www.roguebasin.com/index.php?title=Cellular_Automata_Method_for_Generating_Random_Cave-Like_Levels)
//! * [Autotiling Using Bitmasking](http://www.angryfishstudios.com/2011/04/adventures-in-bitmasking/)
//!
//! This is very slow when built in debug (although it will work just fine),
//! so feel free to build with better optimizations or in release mode.
//!
//! Also, bigger maps tend to look more interesting, so play with the resolution and
//! try out some more interesting tiles to see what you can create.

extern crate ggez;
extern crate rand;

use rand::Rng;
use std::{env, path};

use ggez::{conf, event, graphics};
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::{Context, ContextBuilder, GameResult};

// Some constants for calculated important dimensions.
// TILE_W and TILE_H are determined by the size of the
// tiles in our spritesheet 'caves.png'
const WIN_W: u32 = 800;
const WIN_H: u32 = 600;
const TILE_W: f32 = 20.0;
const TILE_H: f32 = 20.0;
const MAP_W: usize = (WIN_W / TILE_W as u32) as usize;
const MAP_H: usize = (WIN_H / TILE_H as u32) as usize;

// This simple struct will track the width and height
// of the map and use two Vectors for determining each
// generation that we simulate.
struct CellularAutomata {
    w: usize,
    h: usize,
    cells: Vec<Vec<u8>>,
    next: Vec<Vec<u8>>,
}

impl CellularAutomata {
    // A new CellularAutomata will randomly place 'live' tiles
    // and simulate a few generations in order to get a nice
    // looking cave.
    fn new(w: usize, h: usize) -> CellularAutomata {
        let mut ca = CellularAutomata {
            w: w,
            h: h,
            cells: vec![vec![0; h]; w],
            next: vec![vec![0; h]; w],
        };

        // The chance of a tile being 'live'
        // Tweaking this will affect final cave results.
        let chance = 40;
        let mut rng = rand::thread_rng();
        for row in 0..MAP_W {
            for col in 0..MAP_H {
                if (rng.gen::<u32>() % 100) < chance {
                    ca.cells[row][col] = 1;
                }
            }
        }

        // I like to use two passes to get nice caves. Check
        // out the 'step' function below to figure out how you
        // can tweak the numbers to modify the cave results.
        for _ in 0..5 {
            ca.step(4, 1);
        }

        for _ in 0..4 {
            ca.step(4, 0);
        }

        ca
    }

    // The bitmask technique simple adds up values for
    // each adjacent tile in the 4 cardinal directions.
    // The sum will be used to index the tile that will
    // be drawn.
    fn bitmask(&self, x: usize, y: usize) -> u8 {
        let mut sum = 0;

        // up
        if y != 0 {
            sum += self.cells[x][y - 1] * 1;
        }

        // right
        if (x + 1) < self.w {
            sum += self.cells[x + 1][y] * 2;
        }

        // down
        if (y + 1) < self.h {
            sum += self.cells[x][y + 1] * 4;
        }

        // left
        if x != 0 {
            sum += self.cells[x - 1][y] * 8;
        }

        sum
    }

    // The Cellular Automata rules we use are based on number of
    // neighbors. This helper function gets us that number.
    fn count_neighbors(&self, x: usize, y: usize, dist: u32) -> u8 {
        let top = y as i32 - dist as i32;
        let bot = y as i32 + dist as i32 + 1;
        let left = x as i32 - dist as i32;
        let right = x as i32 + dist as i32 + 1;

        let mut count = 0;
        for i in left..right {
            for j in top..bot {
                if i < 0 || i >= self.w as i32 {
                    count += 1;
                    continue;
                }

                if j < 0 || j >= self.h as i32 {
                    count += 1;
                    continue;
                }

                count += self.cells[i as usize][j as usize];
            }
        }

        count
    }

    // High and low are exclusive so I could turn off low when it is 0
    fn step(&mut self, high: u8, low: u8) {
        for row in 0..self.w {
            for col in 0..self.h {
                let ct = self.count_neighbors(row, col, 1);
                let ct2 = self.count_neighbors(row, col, 3);

                // Tile is live if neighbors are within bounds.
                if ct > high || ct2 < low {
                    self.next[row][col] = 1;
                } else {
                    self.next[row][col] = 0;
                }
            }
        }

        // Copy from next back into cells.
        for row in 0..self.w {
            for col in 0..self.h {
                self.cells[row][col] = self.next[row][col];
            }
        }
    }
}

// First we make a structure to contain the game's state
struct MainState {
    img_w: f32,
    img_h: f32,
    sprites: SpriteBatch,
    frames: usize,
    ca: CellularAutomata,
}

// Then we implement the `ggez:event::EventHandler` trait on it, which
// requires callbacks for updating and drawing the game state each frame.
//
// The `EventHandler` trait also contains callbacks for event handling
// that you can override if you wish, but the defaults are fine.
impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        // The spritebatch will hold the spritesheet
        let img = graphics::Image::new(ctx, "/caves.png")?;
        let iw = img.width() as f32;
        let ih = img.height() as f32;
        let mut sprites = SpriteBatch::new(img);

        // Iterating over the CellularAutomata cells will determine
        // the src and dest rectangles for drawing each tile.
        let ca = CellularAutomata::new(MAP_W, MAP_H);
        for row in 0..ca.w {
            for col in 0..ca.h {
                if ca.cells[row][col] == 1 {
                    // Get the index for the sprite to draw
                    let idx = ca.bitmask(row, col);

                    let w = (iw / TILE_W as f32) as u8;
                    let x = (idx % w) as f32;
                    let y = (idx / w) as f32;

                    let param = graphics::DrawParam {
                        src: graphics::Rect::new(
                            x * TILE_W / iw,
                            y * TILE_H / ih,
                            TILE_W / iw,
                            TILE_H / ih,
                        ),
                        dest: graphics::Point2::new(row as f32 * TILE_W, col as f32 * TILE_H),
                        ..Default::default()
                    };

                    // Adding the parameters to the spritesheet means
                    // we can batch draw calls, improving performance.
                    sprites.add(param);
                }
            }
        }

        let state = MainState {
            img_w: iw,
            img_h: ih,
            sprites: sprites,
            frames: 0,
            ca: ca,
        };
        Ok(state)
    }

    // In order to continuously draw new caves, we make
    // a helper function that we run periodically.
    fn new_map(&mut self) {
        self.sprites.clear();

        self.ca = CellularAutomata::new(MAP_W, MAP_H);
        for row in 0..self.ca.w {
            for col in 0..self.ca.h {
                if self.ca.cells[row][col] == 1 {
                    let idx = self.ca.bitmask(row, col);
                    let w = (self.img_w / TILE_W as f32) as u8;
                    let x = (idx % w) as f32;
                    let y = (idx / w) as f32;
                    let param = graphics::DrawParam {
                        src: graphics::Rect::new(
                            x * TILE_W / self.img_w,
                            y * TILE_H / self.img_h,
                            TILE_W / self.img_w,
                            TILE_H / self.img_h,
                        ),
                        dest: graphics::Point2::new(row as f32 * TILE_W, col as f32 * TILE_H),
                        ..Default::default()
                    };

                    self.sprites.add(param);
                }
            }
        }
    }
}

impl event::EventHandler for MainState {
    // In update, we determine how often we should draw a new cave.
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.frames += 1;
        if (self.frames % 60) == 0 {
            self.new_map();
            println!("FPS: {}", ggez::timer::get_fps(ctx));
        }

        Ok(())
    }

    // Having a SpriteBatch makes life easy. Just pass it into graphics::draw
    // and add a few 0-values that won't affect the SpriteBatch.
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        graphics::draw(ctx, &self.sprites, graphics::Point2::new(0.0, 0.0), 0.0)?;
        graphics::present(ctx);
        Ok(())
    }
}

// Now our main function, which does three things:
//
// * First, create a new ggez::ContextBuilder so that we can
// specify the window size using WIN_W and WIN_H.
// * Next, setup the path needed to find any resource files.
// * Lastly, build the contex and us it to create a new MainState
// which will kick off the game loop.
fn main() {
    let mut cb = ContextBuilder::new("Cellular Automata Generator", "ggez")
        .window_setup(conf::WindowSetup::default().title("Cellular Automata Gen!"))
        .window_mode(conf::WindowMode::default().dimensions(WIN_W, WIN_H));

    // We add the CARGO_MANIFEST_DIR/resources do the filesystems paths so
    // we we look in the cargo project for files.
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        cb = cb.add_resource_path(path);
    }

    let ctx = &mut cb.build().unwrap();
    let state = &mut MainState::new(ctx).unwrap();

    if let Err(e) = event::run(ctx, state) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited normally");
    }
}
