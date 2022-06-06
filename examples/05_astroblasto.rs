//! An Asteroids-ish example game to show off ggez.
//! The idea is that this game is simple but still
//! non-trivial enough to be interesting.

use ggez::audio;
use ggez::audio::SoundSource;
use ggez::conf;
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color};
use ggez::input::keyboard::KeyCode;
use ggez::timer;
use ggez::{Context, ContextBuilder, GameResult};
use glam::*;
use oorandom::Rand32;

use ggez::input::keyboard::KeyInput;
use std::env;
use std::path;

type Point2 = Vec2;
type Vector2 = Vec2;

/// *********************************************************************
/// Basic stuff, make some helpers for vector functions.
/// We use the glam math library to provide lots of
/// math stuff.  This just adds some helpers.
/// **********************************************************************

/// Create a unit vector representing the
/// given angle (in radians)
fn vec_from_angle(angle: f32) -> Vector2 {
    let vx = angle.sin();
    let vy = angle.cos();
    Vector2::new(vx, vy)
}

/// Makes a random `Vector2` with the given max magnitude.
fn random_vec(rng: &mut Rand32, max_magnitude: f32) -> Vector2 {
    let angle = rng.rand_float() * 2.0 * std::f32::consts::PI;
    let mag = rng.rand_float() * max_magnitude;
    vec_from_angle(angle) * (mag)
}

/// *********************************************************************
/// Now we define our Actors.
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
    velocity: Vector2,
    ang_vel: f32,
    bbox_size: f32,

    // I am going to lazily overload "life" with a
    // double meaning:
    // for shots, it is the time left to live,
    // for players and rocks, it is the actual hit points.
    life: f32,
}

const PLAYER_LIFE: f32 = 1.0;
const SHOT_LIFE: f32 = 2.0;
const ROCK_LIFE: f32 = 1.0;

const PLAYER_BBOX: f32 = 12.0;
const ROCK_BBOX: f32 = 12.0;
const SHOT_BBOX: f32 = 6.0;

const MAX_ROCK_VEL: f32 = 50.0;

/// *********************************************************************
/// Now we have some constructor functions for different game objects.
/// **********************************************************************

fn create_player() -> Actor {
    Actor {
        tag: ActorType::Player,
        pos: Point2::ZERO,
        facing: 0.,
        velocity: Vector2::ZERO,
        ang_vel: 0.,
        bbox_size: PLAYER_BBOX,
        life: PLAYER_LIFE,
    }
}

fn create_rock() -> Actor {
    Actor {
        tag: ActorType::Rock,
        pos: Point2::ZERO,
        facing: 0.,
        velocity: Vector2::ZERO,
        ang_vel: 0.,
        bbox_size: ROCK_BBOX,
        life: ROCK_LIFE,
    }
}

fn create_shot() -> Actor {
    Actor {
        tag: ActorType::Shot,
        pos: Point2::ZERO,
        facing: 0.,
        velocity: Vector2::ZERO,
        ang_vel: SHOT_ANG_VEL,
        bbox_size: SHOT_BBOX,
        life: SHOT_LIFE,
    }
}

/// Create the given number of rocks.
/// Makes sure that none of them are within the
/// given exclusion zone (nominally the player)
/// Note that this *could* create rocks outside the
/// bounds of the playing field, so it should be
/// called before `wrap_actor_position()` happens.
fn create_rocks(
    rng: &mut Rand32,
    num: i32,
    exclusion: Point2,
    min_radius: f32,
    max_radius: f32,
) -> Vec<Actor> {
    assert!(max_radius > min_radius);
    let new_rock = |_| {
        let mut rock = create_rock();
        let r_angle = rng.rand_float() * 2.0 * std::f32::consts::PI;
        let r_distance = rng.rand_float() * (max_radius - min_radius) + min_radius;
        rock.pos = exclusion + vec_from_angle(r_angle) * r_distance;
        rock.velocity = random_vec(rng, MAX_ROCK_VEL);
        rock
    };
    (0..num).map(new_rock).collect()
}

/// *********************************************************************
/// Now we make functions to handle physics.  We do simple Newtonian
/// physics (so we do have inertia), and cap the max speed so that we
/// don't have to worry too much about small objects clipping through
/// each other.
///
/// Our unit of world space is simply pixels, though we do transform
/// the coordinate system so that +y is up and -y is down.
/// **********************************************************************

/// How fast shots move.
const SHOT_SPEED: f32 = 200.0;
/// Angular velocity of how fast shots rotate.
const SHOT_ANG_VEL: f32 = 0.1;

/// Acceleration in pixels per second.
const PLAYER_THRUST: f32 = 100.0;
/// Rotation in radians per second.
const PLAYER_TURN_RATE: f32 = 3.0;
/// Refire delay between shots, in seconds.
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
    // Clamp the velocity to the max efficiently
    let norm_sq = actor.velocity.length_squared();
    if norm_sq > MAX_PHYSICS_VEL.powi(2) {
        actor.velocity = actor.velocity / norm_sq.sqrt() * MAX_PHYSICS_VEL;
    }
    let dv = actor.velocity * dt;
    actor.pos += dv;
    actor.facing += actor.ang_vel;
}

/// Takes an actor and wraps its position to the bounds of the
/// screen, so if it goes off the left side of the screen it
/// will re-enter on the right side and so on.
fn wrap_actor_position(actor: &mut Actor, sx: f32, sy: f32) {
    // Wrap screen
    let screen_x_bounds = sx / 2.0;
    let screen_y_bounds = sy / 2.0;
    if actor.pos.x > screen_x_bounds {
        actor.pos -= Vec2::new(sx, 0.0);
    } else if actor.pos.x < -screen_x_bounds {
        actor.pos += Vec2::new(sx, 0.0);
    };
    if actor.pos.y > screen_y_bounds {
        actor.pos -= Vec2::new(0.0, sy);
    } else if actor.pos.y < -screen_y_bounds {
        actor.pos += Vec2::new(0.0, sy);
    }
}

fn handle_timed_life(actor: &mut Actor, dt: f32) {
    actor.life -= dt;
}

/// Translates the world coordinate system, which
/// has Y pointing up and the origin at the center,
/// to the screen coordinate system, which has Y
/// pointing downward and the origin at the top-left,
fn world_to_screen_coords(screen_width: f32, screen_height: f32, point: Point2) -> Point2 {
    let x = point.x + screen_width / 2.0;
    let y = screen_height - (point.y + screen_height / 2.0);
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
    shot_sound: audio::Source,
    hit_sound: audio::Source,
}

impl Assets {
    fn new(ctx: &mut Context) -> GameResult<Assets> {
        let player_image = graphics::Image::from_path(ctx, "/player.png", true)?;
        let shot_image = graphics::Image::from_path(ctx, "/shot.png", true)?;
        let rock_image = graphics::Image::from_path(ctx, "/rock.png", true)?;

        let shot_sound = audio::Source::new(ctx, "/pew.ogg")?;
        let hit_sound = audio::Source::new(ctx, "/boom.ogg")?;

        Ok(Assets {
            player_image,
            shot_image,
            rock_image,
            shot_sound,
            hit_sound,
        })
    }

    fn actor_image(&self, actor: &Actor) -> &graphics::Image {
        match actor.tag {
            ActorType::Player => &self.player_image,
            ActorType::Rock => &self.rock_image,
            ActorType::Shot => &self.shot_image,
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
/// We simply keep game objects in a vector for each actor type, and we
/// probably mingle gameplay-state (like score) and hardware-state
/// (like `input`) a little more than we should, but for something
/// this small it hardly matters.
/// **********************************************************************

struct MainState {
    // because we want to screenshot, we need to ensure we're rendering to Rgba8
    screen: graphics::ScreenImage,
    player: Actor,
    shots: Vec<Actor>,
    rocks: Vec<Actor>,
    level: i32,
    score: i32,
    assets: Assets,
    screen_width: f32,
    screen_height: f32,
    input: InputState,
    player_shot_timeout: f32,
    rng: Rand32,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        println!("Game resource path: {:?}", ctx.fs);

        print_instructions();

        // Seed our RNG
        let mut seed: [u8; 8] = [0; 8];
        getrandom::getrandom(&mut seed[..]).expect("Could not create RNG seed");
        let mut rng = Rand32::new(u64::from_ne_bytes(seed));

        let assets = Assets::new(ctx)?;
        let player = create_player();
        let rocks = create_rocks(&mut rng, 5, player.pos, 100.0, 250.0);

        let (width, height) = ctx.gfx.drawable_size();
        let screen =
            graphics::ScreenImage::new(ctx, graphics::ImageFormat::Rgba8UnormSrgb, 1., 1., 1);

        let s = MainState {
            screen,
            player,
            shots: Vec::new(),
            rocks,
            level: 0,
            score: 0,
            assets,
            screen_width: width,
            screen_height: height,
            input: InputState::default(),
            player_shot_timeout: 0.0,
            rng,
        };

        Ok(s)
    }

    fn fire_player_shot(&mut self, ctx: &Context) -> GameResult {
        self.player_shot_timeout = PLAYER_SHOT_TIME;

        let player = &self.player;
        let mut shot = create_shot();
        shot.pos = player.pos;
        shot.facing = player.facing;
        let direction = vec_from_angle(shot.facing);
        shot.velocity = SHOT_SPEED * direction;

        self.shots.push(shot);

        self.assets.shot_sound.play(ctx)?;

        Ok(())
    }

    fn clear_dead_stuff(&mut self) {
        self.shots.retain(|s| s.life > 0.0);
        self.rocks.retain(|r| r.life > 0.0);
    }

    fn handle_collisions(&mut self, ctx: &Context) -> GameResult {
        for rock in &mut self.rocks {
            let pdistance = rock.pos - self.player.pos;
            if pdistance.length() < (self.player.bbox_size + rock.bbox_size) {
                self.player.life = 0.0;
            }
            for shot in &mut self.shots {
                let distance = shot.pos - rock.pos;
                if distance.length() < (shot.bbox_size + rock.bbox_size) {
                    shot.life = 0.0;
                    rock.life = 0.0;
                    self.score += 1;

                    self.assets.hit_sound.play(ctx)?;
                }
            }
        }
        Ok(())
    }

    fn check_for_level_respawn(&mut self) {
        if self.rocks.is_empty() {
            self.level += 1;
            let r = create_rocks(&mut self.rng, self.level + 5, self.player.pos, 100.0, 250.0);
            self.rocks.extend(r);
        }
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

fn draw_actor(
    assets: &mut Assets,
    canvas: &mut graphics::Canvas,
    actor: &Actor,
    world_coords: (f32, f32),
) {
    let (screen_w, screen_h) = world_coords;
    let pos = world_to_screen_coords(screen_w, screen_h, actor.pos);
    let image = assets.actor_image(actor);
    let drawparams = graphics::DrawParam::new()
        .dest(pos)
        .rotation(actor.facing as f32)
        .offset(Point2::new(0.5, 0.5));
    canvas.draw(image, drawparams);
}

/// **********************************************************************
/// Now we implement the `EventHandler` trait from `ggez::event`, which provides
/// ggez with callbacks for updating and drawing our game, as well as
/// handling input events.
/// **********************************************************************
impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;

        while ctx.time.check_update_time(DESIRED_FPS) {
            let seconds = 1.0 / (DESIRED_FPS as f32);

            // Update the player state based on the user input.
            player_handle_input(&mut self.player, &self.input, seconds);
            self.player_shot_timeout -= seconds;
            if self.input.fire && self.player_shot_timeout < 0.0 {
                self.fire_player_shot(ctx)?;
            }

            // Update the physics for all actors.
            // First the player...
            update_actor_position(&mut self.player, seconds);
            wrap_actor_position(
                &mut self.player,
                self.screen_width as f32,
                self.screen_height as f32,
            );

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
            self.handle_collisions(ctx)?;

            self.clear_dead_stuff();

            self.check_for_level_respawn();

            // Finally we check for our end state.
            // I want to have a nice death screen eventually,
            // but for now we just quit.
            if self.player.life <= 0.0 {
                println!("Game over!");
                let _ = event::request_quit(ctx);
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // Our drawing is quite simple.
        // Just clear the screen...
        let mut canvas = graphics::Canvas::from_screen_image(ctx, &mut self.screen, Color::BLACK);

        // Loop over all objects drawing them...
        {
            let assets = &mut self.assets;
            let coords = (self.screen_width, self.screen_height);

            let p = &self.player;
            draw_actor(assets, &mut canvas, p, coords);

            for s in &self.shots {
                draw_actor(assets, &mut canvas, s, coords);
            }

            for r in &self.rocks {
                draw_actor(assets, &mut canvas, r, coords);
            }
        }

        // And draw the GUI elements in the right places.
        let level_dest = Point2::new(10.0, 10.0);
        let score_dest = Point2::new(200.0, 10.0);

        let level_str = format!("Level: {}", self.level);
        let score_str = format!("Score: {}", self.score);

        canvas.draw(
            &graphics::Text::new(level_str),
            graphics::DrawParam::from(level_dest).color(Color::WHITE),
        );

        canvas.draw(
            &graphics::Text::new(score_str),
            graphics::DrawParam::from(score_dest).color(Color::WHITE),
        );

        canvas.finish(ctx)?;
        ctx.gfx.present(&self.screen.image(ctx))?;

        // And yield the timeslice
        // This tells the OS that we're done using the CPU but it should
        // get back to this program as soon as it can.
        // This ideally prevents the game from using 100% CPU all the time
        // even if vsync is off.
        // The actual behavior can be a little platform-specific.
        timer::yield_now();
        Ok(())
    }

    // Handle key events.  These just map keyboard events
    // and alter our input state appropriately.
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: KeyInput,
        _repeated: bool,
    ) -> GameResult {
        match input.keycode {
            Some(KeyCode::Up) => {
                self.input.yaxis = 1.0;
            }
            Some(KeyCode::Left) => {
                self.input.xaxis = -1.0;
            }
            Some(KeyCode::Right) => {
                self.input.xaxis = 1.0;
            }
            Some(KeyCode::Space) => {
                self.input.fire = true;
            }
            Some(KeyCode::P) => {
                self.screen.image(ctx).encode(
                    ctx,
                    graphics::ImageEncodingFormat::Png,
                    "/screenshot.png",
                )?;
            }
            Some(KeyCode::Escape) => event::request_quit(ctx),
            _ => (), // Do nothing
        }
        Ok(())
    }

    fn key_up_event(&mut self, _ctx: &mut Context, input: KeyInput) -> GameResult {
        match input.keycode {
            Some(KeyCode::Up) => {
                self.input.yaxis = 0.0;
            }
            Some(KeyCode::Left | KeyCode::Right) => {
                self.input.xaxis = 0.0;
            }
            Some(KeyCode::Space) => {
                self.input.fire = false;
            }
            _ => (), // Do nothing
        }
        Ok(())
    }
}

/// **********************************************************************
/// Finally our main function!  Which merely sets up a config and calls
/// `ggez::event::run()` with our `EventHandler` type.
/// **********************************************************************

pub fn main() -> GameResult {
    // We add the CARGO_MANIFEST_DIR/resources to the resource paths
    // so that ggez will look in our cargo project directory for files.
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ContextBuilder::new("astroblasto", "ggez")
        .window_setup(conf::WindowSetup::default().title("Astroblasto!"))
        .window_mode(conf::WindowMode::default().dimensions(640.0, 480.0))
        .add_resource_path(resource_dir);

    let (mut ctx, events_loop) = cb.build()?;

    let game = MainState::new(&mut ctx)?;
    event::run(ctx, events_loop, game)
}
