//! An Asteroids-ish example game to show off ggez.
//! The idea is that this game is simple but still
//! non-trivial enough to be interesting.

extern crate ggez;
extern crate rand;

use ggez::audio;
use ggez::conf;
use ggez::event::*;
use ggez::{Context, GameResult};
use ggez::graphics;
use ggez::timer;
use std::time::Duration;

/// *********************************************************************
/// Basic stuff.
/// First, we create a vector type.
/// You're probably better off using a real vector math lib but I
/// didn't want to add more dependencies and such.
/// **********************************************************************

use ggez::nalgebra as na;
use ggez::graphics::Point as Point2;
use ggez::graphics::Vector as Vec2;

// #[derive(Debug, Copy, Clone)]
// struct Vec2 {
//     x: f32,
//     y: f32,
// }

// impl Vec2 {
//     fn new(x: f32, y: f32) -> Self {
//         Vec2 { x: x, y: y }
//     }

//     /// Create a unit vector representing the
//     /// given angle (in radians)
fn vec_from_angle(angle: f32) -> Vec2 {
    let vx = angle.sin();
    let vy = angle.cos();
    Vec2::new(vx, vy)
}

// fn point_from_angle(angle: f32) -> Point2 {
//     let vx = angle.sin();
//     let vy = angle.cos();
//     Point2::new(vx, vy)
// }

fn random_vec(max_magnitude: f32) -> Vec2 {
    let angle = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
    let mag = rand::random::<f32>() * max_magnitude;
    vec_from_angle(angle) * (mag)
}


// fn random_point(max_magnitude: f32) -> Point2 {
//     let angle = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
//     let mag = rand::random::<f32>() * max_magnitude;
//     point_from_angle(angle) * (mag)
// }

//     fn magnitude(&self) -> f32 {
//         ((self.x * self.x) + (self.y * self.y)).sqrt()
//     }

//     fn normalized(&self) -> Self {
//         let mag = self.magnitude();
//         self.scaled(1.0 / mag)
//     }

//     fn scaled(&self, rhs: f32) -> Self {
//         Vec2 {
//             x: self.x * rhs,
//             y: self.y * rhs,
//         }
//     }

//     /// Returns a vector whose magnitude is between
//     /// 0 and max.
//     fn clamped(&self, max: f32) -> Self {
//         let mag = self.magnitude();
//         if mag > max {
//             self.normalized().scaled(max)
//         } else {
//             *self
//         }
//     }
// }

// impl Add for Vec2 {
//     type Output = Self;
//     fn add(self, rhs: Vec2) -> Self {
//         Vec2 {
//             x: self.x + rhs.x,
//             y: self.y + rhs.y,
//         }
//     }
// }


// impl AddAssign for Vec2 {
//     fn add_assign(&mut self, rhs: Vec2) {
//         self.x += rhs.x;
//         self.y += rhs.y;
//     }
// }


// impl Sub for Vec2 {
//     type Output = Self;
//     fn sub(self, rhs: Vec2) -> Self {
//         Vec2 {
//             x: self.x - rhs.x,
//             y: self.y - rhs.y,
//         }
//     }
// }

// impl Default for Vec2 {
//     fn default() -> Self {
//         Self::new(0., 0.)
//     }
// }

/// *********************************************************************
/// Now we define our Actor's.
/// An Actor is anything in the game world.
/// We're not *quite* making a real entity-component system but it's
/// pretty close.  For a more complicated game you would want a
/// real ECS, but for this it's enough to say that all our game objects
/// contain pretty much the same data.
/// **********************************************************************
#[derive(Debug)]
enum ActorType {
    Player,
    Rock,
    Shot,
}

#[derive(Debug)]
struct Actor {
    tag: ActorType,
    pos: Point2,
    facing: f32,
    velocity: Vec2,
    rvel: f32,
    bbox_size: f32,

    // I am going to lazily overload "life" with a
    // double meaning rather than making a proper ECS;
    // for shots, it is the time left to live,
    // for players and such, it is the actual hit points.
    life: f32,
}

const PLAYER_LIFE: f32 = 1.0;
const SHOT_LIFE: f32 = 2.0;
const ROCK_LIFE: f32 = 1.0;

const PLAYER_BBOX: f32 = 12.0;
const ROCK_BBOX: f32 = 12.0;
const SHOT_BBOX: f32 = 6.0;

/// *********************************************************************
/// Now we have some initializer functions for different game objects.
/// **********************************************************************

fn create_player() -> Actor {
    Actor {
        tag: ActorType::Player,
        pos: Point2::origin(),
        facing: 0.,
        velocity: na::zero(),
        rvel: 0.,
        bbox_size: PLAYER_BBOX,
        life: PLAYER_LIFE,
    }
}

fn create_rock() -> Actor {
    Actor {
        tag: ActorType::Rock,
        pos: Point2::origin(),
        facing: 0.,
        velocity: na::zero(),
        rvel: 0.,
        bbox_size: ROCK_BBOX,
        life: ROCK_LIFE,
    }
}

fn create_shot() -> Actor {
    Actor {
        tag: ActorType::Shot,
        pos: Point2::origin(),
        facing: 0.,
        velocity: na::zero(),
        rvel: SHOT_RVEL,
        bbox_size: SHOT_BBOX,
        life: SHOT_LIFE,
    }
}

const MAX_ROCK_VEL: f32 = 50.0;

/// Create the given number of rocks.
/// Makes sure that none of them are within the
/// given exclusion zone (nominally the player)
/// Note that this *could* create rocks outside the
/// bounds of the playing field, so it should be
/// called before `wrap_actor_position()` happens.
fn create_rocks(num: i32, exclusion: &Point2, min_radius: f32, max_radius: f32) -> Vec<Actor> {
    assert!(max_radius > min_radius);
    let new_rock = |_| {
        let mut rock = create_rock();
        let r_angle = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
        let r_distance = rand::random::<f32>() * (max_radius - min_radius) + min_radius;
        rock.pos = *exclusion + vec_from_angle(r_angle) * r_distance;
        rock.velocity = random_vec(MAX_ROCK_VEL);
        rock
    };
    (0..num).map(new_rock).collect()
}

/// *********************************************************************
/// Now we have functions to handle physics.  We do simple Newtonian
/// physics (so we do have inertia), and cap the max speed so that we
/// don't have to worry too much about small objects clipping through
/// each other.
///
/// Our unit of world space is simply pixels, though we do transform
/// the coordinate system so that +y is up and -y is down.
/// **********************************************************************

const SHOT_SPEED: f32 = 200.0;
const SHOT_RVEL: f32 = 0.1;
const SPRITE_SIZE: u32 = 32;

// Acceleration in pixels per second, more or less.
const PLAYER_THRUST: f32 = 100.0;
// Rotation in radians per second.
const PLAYER_TURN_RATE: f32 = 3.05;
// Seconds between shots
const PLAYER_SHOT_TIME: f32 = 0.5;


fn player_handle_input(actor: &mut Actor, input: &InputState, dt: f32) {
    actor.facing += dt * PLAYER_TURN_RATE * input.xaxis;

    if input.yaxis > 0.0 {
        player_thrust(actor, dt);
    }
}

fn player_thrust(actor: &mut Actor, dt: f32) {
    let direction_vector = vec_from_angle(actor.facing);
    let thrust_vector = direction_vector * (PLAYER_THRUST);
    actor.velocity += thrust_vector * (dt);
}

const MAX_PHYSICS_VEL: f32 = 250.0;

fn update_actor_position(actor: &mut Actor, dt: f32) {
    let norm_sq = actor.velocity.norm_squared();
    if norm_sq > MAX_PHYSICS_VEL.powi(2) {
        actor.velocity = actor.velocity / norm_sq.sqrt() * MAX_PHYSICS_VEL;
    }
    let dv = actor.velocity * (dt);
    actor.pos += dv;
    actor.facing += actor.rvel;
}

/// Takes an actor and wraps its position to the bounds of the
/// screen, so if it goes off the left side of the screen it
/// will re-enter on the right side and so on.
fn wrap_actor_position(actor: &mut Actor, sx: f32, sy: f32) {
    // Wrap screen
    let screen_x_bounds = sx / 2.0;
    let screen_y_bounds = sy / 2.0;
    let sprite_half_size = (SPRITE_SIZE / 2) as f32;
    let actor_center = actor.pos - Vec2::new(-sprite_half_size, sprite_half_size);
    if actor_center.x > screen_x_bounds {
        actor.pos.x -= sx;
    } else if actor_center.x < -screen_x_bounds {
        actor.pos.x += sx;
    };
    if actor_center.y > screen_y_bounds {
        actor.pos.y -= sy;
    } else if actor_center.y < -screen_y_bounds {
        actor.pos.y += sy;
    }
}

fn handle_timed_life(actor: &mut Actor, dt: f32) {
    actor.life -= dt;
}


/// Translates the world coordinate system, which
/// has Y pointing up and the origin at the center,
/// to the screen coordinate system, which has Y
/// pointing downward and the origin at the top-left,
fn world_to_screen_coords(screen_width: u32, screen_height: u32, point: &Point2) -> Point2 {
    let width = screen_width as f32;
    let height = screen_height as f32;
    let x = point.x + width / 2.0;
    let y = height - (point.y + height / 2.0);
    Point2::new(x, y)
}

/// **********************************************************************
/// So that was the real meat of our game.  Now we just need a structure
/// to contain the images, sounds, etc. that we need to hang on to; this
/// is our "asset management system".  All the file names and such are
/// just hard-coded.
/// **********************************************************************

struct Assets {
    player_image: graphics::Image,
    shot_image: graphics::Image,
    rock_image: graphics::Image,
    font: graphics::Font,
    shot_sound: audio::Source,
    hit_sound: audio::Source,
}

impl Assets {
    fn new(ctx: &mut Context) -> GameResult<Assets> {
        let player_image = graphics::Image::new(ctx, "/player.png")?;
        let shot_image = graphics::Image::new(ctx, "/shot.png")?;
        let rock_image = graphics::Image::new(ctx, "/rock.png")?;
        // let font_path = path::Path::new("/consolefont.png");
        // let font_chars =
        //"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!,.?;'\"";
        // let font = graphics::Font::new_bitmap(ctx, font_path, font_chars)?;
        let font = graphics::Font::new(ctx, "/DejaVuSerif.ttf", 18)?;

        let shot_sound = audio::Source::new(ctx, "/pew.ogg")?;
        let hit_sound = audio::Source::new(ctx, "/boom.ogg")?;
        Ok(Assets {
               player_image: player_image,
               shot_image: shot_image,
               rock_image: rock_image,
               font: font,
               shot_sound: shot_sound,
               hit_sound: hit_sound,
           })
    }

    fn actor_image(&mut self, actor: &Actor) -> &mut graphics::Image {
        match actor.tag {
            ActorType::Player => &mut self.player_image,
            ActorType::Rock => &mut self.rock_image,
            ActorType::Shot => &mut self.shot_image,
        }
    }
}

/// **********************************************************************
/// The `InputState` is exactly what it sounds like, it just keeps track of
/// the user's input state so that we turn keyboard events into something
/// state-based and device-independent.
/// **********************************************************************
#[derive(Debug)]
struct InputState {
    xaxis: f32,
    yaxis: f32,
    fire: bool,
}

impl Default for InputState {
    fn default() -> Self {
        InputState {
            xaxis: 0.0,
            yaxis: 0.0,
            fire: false,
        }
    }
}

/// **********************************************************************
/// Now we're getting into the actual game loop.  The `MainState` is our
/// game's "global" state, it keeps track of everything we need for
/// actually running the game.
///
/// Our game objects are simply a vector for each actor type, and we
/// probably mingle gameplay-state (like score) and hardware-state
/// (like gui_`dirty`) a little more than we should, but for something
/// this small it hardly matters.
/// **********************************************************************

struct MainState {
    player: Actor,
    shots: Vec<Actor>,
    rocks: Vec<Actor>,
    level: i32,
    score: i32,
    assets: Assets,
    screen_width: u32,
    screen_height: u32,
    input: InputState,
    player_shot_timeout: f32,
    gui_dirty: bool,
    score_display: graphics::Text,
    level_display: graphics::Text,
}


impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        ctx.print_resource_stats();
        graphics::set_background_color(ctx, (0, 0, 0, 255).into());

        println!("Game resource path: {:?}", ctx.filesystem);

        print_instructions();

        let assets = Assets::new(ctx)?;
        let score_disp = graphics::Text::new(ctx, "score", &assets.font)?;
        let level_disp = graphics::Text::new(ctx, "level", &assets.font)?;

        let player = create_player();
        let rocks = create_rocks(5, &player.pos, 100.0, 250.0);

        let s = MainState {
            player: player,
            shots: Vec::new(),
            rocks: rocks,
            level: 0,
            score: 0,
            assets: assets,
            screen_width: ctx.conf.window_width,
            screen_height: ctx.conf.window_height,
            input: InputState::default(),
            player_shot_timeout: 0.0,
            gui_dirty: true,
            score_display: score_disp,
            level_display: level_disp,
        };

        Ok(s)
    }

    fn fire_player_shot(&mut self) {
        self.player_shot_timeout = PLAYER_SHOT_TIME;

        let player = &self.player;
        let mut shot = create_shot();
        shot.pos = player.pos;
        shot.facing = player.facing;
        let direction = vec_from_angle(shot.facing);
        shot.velocity.x = SHOT_SPEED * direction.x;
        shot.velocity.y = SHOT_SPEED * direction.y;

        self.shots.push(shot);
        let _ = self.assets.shot_sound.play();
    }



    fn clear_dead_stuff(&mut self) {
        self.shots.retain(|s| s.life > 0.0);
        self.rocks.retain(|r| r.life > 0.0);
    }

    fn handle_collisions(&mut self) {
        for rock in &mut self.rocks {
            let pdistance = rock.pos - self.player.pos;
            if pdistance.norm() < (self.player.bbox_size + rock.bbox_size) {
                self.player.life = 0.0;
            }
            for shot in &mut self.shots {
                let distance = shot.pos - rock.pos;
                if distance.norm() < (shot.bbox_size + rock.bbox_size) {
                    shot.life = 0.0;
                    rock.life = 0.0;
                    self.score += 1;
                    self.gui_dirty = true;
                    let _ = self.assets.hit_sound.play();
                }
            }
        }
    }

    fn check_for_level_respawn(&mut self) {
        if self.rocks.is_empty() {
            self.level += 1;
            self.gui_dirty = true;
            let r = create_rocks(self.level + 5, &self.player.pos, 100.0, 250.0);
            self.rocks.extend(r);
        }
    }

    fn update_ui(&mut self, ctx: &mut Context) {
        let score_str = format!("Score: {}", self.score);
        let level_str = format!("Level: {}", self.level);
        let score_text = graphics::Text::new(ctx, &score_str, &self.assets.font).unwrap();
        let level_text = graphics::Text::new(ctx, &level_str, &self.assets.font).unwrap();

        self.score_display = score_text;
        self.level_display = level_text;
    }
}


/// **********************************************************************
/// A couple of utility functions.
/// **********************************************************************

fn print_instructions() {
    println!();
    println!("Welcome to ASTROBLASTO!");
    println!();
    println!("How to play:");
    println!("L/R arrow keys rotate your ship, up thrusts, space bar fires");
    println!();
}


fn draw_actor(assets: &mut Assets,
              ctx: &mut Context,
              actor: &Actor,
              world_coords: (u32, u32))
              -> GameResult<()> {
    let (screen_w, screen_h) = world_coords;
    let pos = world_to_screen_coords(screen_w, screen_h, &actor.pos);
    // let pos = Vec2::new(1.0, 1.0);
    let px = pos.x as f32;
    let py = pos.y as f32;
    let dest_point = graphics::Point::new(px, py);
    let image = assets.actor_image(actor);
    graphics::draw(ctx, image, dest_point, actor.facing as f32)

}

/// **********************************************************************
/// Now we implement the `EventHandler` trait from `ggez::event`, which provides
/// ggez with callbacks for updating and drawing our game, as well as
/// handling input events.
/// **********************************************************************
impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context, _dt: Duration) -> GameResult<()> {
        const DESIRED_FPS: u64 = 60;
        if !timer::check_update_time(ctx, DESIRED_FPS) {
            return Ok(());
        }
        let seconds = 1.0 / (DESIRED_FPS as f32);

        // Update the player state based on the user input.
        player_handle_input(&mut self.player, &self.input, seconds);
        self.player_shot_timeout -= seconds;
        if self.input.fire && self.player_shot_timeout < 0.0 {
            self.fire_player_shot();
        }

        // Update the physics for all actors.
        // First the player...
        update_actor_position(&mut self.player, seconds);
        wrap_actor_position(&mut self.player,
                            self.screen_width as f32,
                            self.screen_height as f32);

        // Then the shots...
        for act in &mut self.shots {
            update_actor_position(act, seconds);
            wrap_actor_position(act, self.screen_width as f32, self.screen_height as f32);
            handle_timed_life(act, seconds);
        }

        // And finally the rocks.
        for act in &mut self.rocks {
            update_actor_position(act, seconds);
            wrap_actor_position(act, self.screen_width as f32, self.screen_height as f32);
        }

        // Handle the results of things moving:
        // collision detection, object death, and if
        // we have killed all the rocks in the level,
        // spawn more of them.
        self.handle_collisions();

        self.clear_dead_stuff();

        self.check_for_level_respawn();

        // Using a gui_dirty flag here is a little
        // messy but fine here.
        if self.gui_dirty {
            self.update_ui(ctx);
            self.gui_dirty = false;
        }

        // Finally we check for our end state.
        // I want to have a nice death screen eventually,
        // but for now we just quit.
        if self.player.life <= 0.0 {
            println!("Game over!");
            let _ = ctx.quit();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Our drawing is quite simple.
        // Just clear the screen...
        graphics::clear(ctx);

        // Loop over all objects drawing them...
        {
            let assets = &mut self.assets;
            let coords = (self.screen_width, self.screen_height);

            let p = &self.player;
            draw_actor(assets, ctx, p, coords)?;

            for s in &self.shots {
                draw_actor(assets, ctx, s, coords)?;
            }

            for r in &self.rocks {
                draw_actor(assets, ctx, r, coords)?;
            }
        }


        // And draw the GUI elements in the right places.
        let level_dest = graphics::Point::new((self.level_display.width() / 2) as f32 + 10.0,
                                              (self.level_display.height() / 2) as f32 + 10.0);
        let score_dest = graphics::Point::new((self.score_display.width() / 2) as f32 + 200.0,
                                              (self.score_display.height() / 2) as f32 + 10.0);
        // let source_rect = graphics::Rect::one();
        graphics::draw(ctx, &self.level_display, level_dest, 0.0)?;
        graphics::draw(ctx, &self.score_display, score_dest, 0.0)?;

        // Then we flip the screen...
        graphics::present(ctx);
        // And sleep for 0 seconds.
        // This tells the OS that we're done using the CPU but it should
        // get back to this program as soon as it can.
        // This prevents the game from using 100% CPU all the time
        // even if vsync is off.
        timer::sleep(Duration::from_secs(0));
        Ok(())
    }

    // Handle key events.  These just map keyboard events
    // and alter our input state appropriately.
    fn key_down_event(&mut self, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        match keycode {
            Keycode::Up => {
                self.input.yaxis = 1.0;
            }
            Keycode::Left => {
                self.input.xaxis = -1.0;
            }
            Keycode::Right => {
                self.input.xaxis = 1.0;
            }
            Keycode::Space => {
                self.input.fire = true;
            }
            _ => (), // Do nothing
        }
    }


    fn key_up_event(&mut self, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        match keycode {
            Keycode::Up => {
                self.input.yaxis = 0.0;
            }
            Keycode::Left | Keycode::Right => {
                self.input.xaxis = 0.0;
            }
            Keycode::Space => {
                self.input.fire = false;
            }
            _ => (), // Do nothing
        }
    }
}

/// **********************************************************************
/// Finally our main function!  Which merely sets up a config and calls
/// `ggez::event::run()` with our `EventHandler` type.
/// **********************************************************************

pub fn main() {
    let mut c = conf::Conf::new();
    c.window_title = "Astroblasto!".to_string();
    c.window_width = 640;
    c.window_height = 480;
    c.window_icon = "/player.png".to_string();

    let ctx = &mut Context::load_from_conf("astroblasto", "ggez", c).unwrap();

    match MainState::new(ctx) {
        Err(e) => {
            println!("Could not load game!");
            println!("Error: {}", e);
        }
        Ok(ref mut game) => {
            let result = run(ctx, game);
            if let Err(e) = result {
                println!("Error encountered running game: {}", e);
            } else {
                println!("Game exited cleanly.");
            }
        }
    }
}
