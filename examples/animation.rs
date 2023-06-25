//! An example showcasing animation using `keyframe`.
//! Includes tweening and frame-by-frame animation.
//! Credit for the animation goes to [Dead Revolver](https://deadrevolver.itch.io/pixel-prototype-player-sprites)

#[macro_use]
extern crate num_derive;

use ggez::event;
use ggez::glam::*;
use ggez::graphics::{self, Color};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::mint::Point2;
use ggez::{Context, GameResult};
use keyframe::{ease, functions::*, keyframes, AnimationSequence, EasingFunction};
use keyframe_derive::CanTween;
use num_traits::{FromPrimitive, ToPrimitive};
use std::env;
use std::path;

struct MainState {
    ball: graphics::Mesh,
    spritesheet: graphics::Image,
    easing_enum: EasingEnum,
    animation_type: AnimationType,
    ball_animation: AnimationSequence<Point2<f32>>,
    player_animation: AnimationSequence<TweenableRect>,
    duration: f32,
}

#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq)]
#[repr(i32)]
enum EasingEnum {
    Linear,
    EaseIn,
    EaseInOut,
    EaseOut,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    Bezier,
    EaseInOut3Point,
}

fn easing_function(ease_enum: &EasingEnum) -> Box<dyn EasingFunction + Send + Sync> {
    match ease_enum {
        EasingEnum::Linear => Box::new(Linear),
        EasingEnum::EaseIn => Box::new(EaseIn),
        EasingEnum::EaseInOut => Box::new(EaseInOut),
        EasingEnum::EaseOut => Box::new(EaseOut),
        EasingEnum::EaseInCubic => Box::new(EaseInCubic),
        EasingEnum::EaseOutCubic => Box::new(EaseOutCubic),
        EasingEnum::EaseInOutCubic => Box::new(EaseInOutCubic),
        EasingEnum::Bezier => Box::new(BezierCurve::from([0.6, 0.04].into(), [0.98, 0.335].into())),
        _ => panic!(),
    }
}

fn ball_sequence(ease_enum: &EasingEnum, duration: f32) -> AnimationSequence<Point2<f32>> {
    let ball_pos_start: Point2<f32> = [120.0, 120.0].into();
    let ball_pos_end: Point2<f32> = [120.0, 420.0].into();

    if let EasingEnum::EaseInOut3Point = ease_enum {
        let mid_pos = ease(Linear, ball_pos_start, ball_pos_end, 0.33);
        keyframes![
            (ball_pos_start, 0.0, EaseInOut),
            (mid_pos, 0.66 * duration, EaseInOut), // reach about a third of the height at two thirds of the duration
            (ball_pos_end, duration, EaseInOut)
        ]
    } else {
        keyframes![
            (ball_pos_start, 0.0, easing_function(ease_enum)),
            (ball_pos_end, duration, easing_function(ease_enum)) // this second function is necessary here because the sequence might get reversed
        ]
    }
}

#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq)]
#[repr(i32)]
enum AnimationType {
    Idle,
    Run,
    FrontFlip,
    Roll,
    Crawl,
}

const FRAME_ROWS: i32 = 19;
/// returns the src.y parameter for the animation
fn src_y(anim_type: &AnimationType) -> f32 {
    let row = match anim_type {
        AnimationType::Idle => 1,
        AnimationType::Run => 3,
        AnimationType::FrontFlip => 8,
        AnimationType::Roll => 11,
        AnimationType::Crawl => 10,
    };

    row as f32 / FRAME_ROWS as f32
}

const FRAME_COLUMNS: i32 = 14;
/// returns the final src.x parameter for the last frame of the animation
fn src_x_end(anim_type: &AnimationType) -> f32 {
    (frame_count(anim_type) - 1) as f32 / FRAME_COLUMNS as f32
}

fn frame_count(anim_type: &AnimationType) -> i32 {
    match anim_type {
        AnimationType::Idle => 7,
        AnimationType::Run => 8,
        AnimationType::FrontFlip => 14,
        AnimationType::Roll => 10,
        AnimationType::Crawl => 8,
    }
}

#[derive(CanTween, Clone, Copy)]
/// necessary because we can't implement CanTween for graphics::Rect directly, as it's a foreign type
struct TweenableRect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl TweenableRect {
    fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        TweenableRect { x, y, w, h }
    }
}

impl From<TweenableRect> for graphics::Rect {
    fn from(t_rect: TweenableRect) -> Self {
        graphics::Rect {
            x: t_rect.x,
            y: t_rect.y,
            w: t_rect.w,
            h: t_rect.h,
        }
    }
}

/// A fancy easing function, tweening something into one of `frames` many discrete states.
/// The `pre_easing` is applied first, thereby making other `EasingFunction`s usable in the realm of frame-by-frame animation
struct AnimationFloor {
    pre_easing: Box<dyn EasingFunction + Send + Sync>,
    frames: i32,
}
impl EasingFunction for AnimationFloor {
    #[inline]
    fn y(&self, x: f64) -> f64 {
        (self.pre_easing.y(x) * (self.frames) as f64).floor() / (self.frames - 1) as f64
    }
}

fn player_sequence(
    ease_enum: &EasingEnum,
    anim_type: &AnimationType,
    duration: f32,
) -> AnimationSequence<TweenableRect> {
    // create the two Rects that will serve as `from` and `to` for the DrawParam::src of the animation
    // the start for all animations is at the leftmost frame, starting at 0.0
    let src_x_start: f32 = 0.0;
    // the final parameter depends upon how many frames there are in an animation
    let src_x_end = src_x_end(anim_type);
    // the src.y parameter depends on the row in which the animation is placed inside the sprite sheet
    let src_y = src_y(anim_type);
    // the height and width of the source rect are the proportions of a frame relative towards the whole sprite sheet
    let w = 1.0 / FRAME_COLUMNS as f32;
    let h = 1.0 / FRAME_ROWS as f32;
    let src_rect_start = TweenableRect::new(src_x_start, src_y, w, h);
    let src_end_rect = TweenableRect::new(src_x_end, src_y, w, h);

    let frames = frame_count(anim_type);

    if let EasingEnum::EaseInOut3Point = ease_enum {
        // first calculate the middle state of this sequence
        // luckily we can use keyframe to help us with that
        let mid = ease(
            AnimationFloor {
                pre_easing: Box::new(Linear),
                frames,
            },
            src_rect_start,
            src_end_rect,
            0.33,
        );
        let mid_frames = (frames as f32 * 0.33).floor() as i32;
        // we need to adapt the frame count for each keyframe
        // only the frames that are to be played until the next keyframe count
        keyframes![
            (
                src_rect_start,
                0.0,
                AnimationFloor {
                    pre_easing: Box::new(EaseInOut),
                    frames: mid_frames + 1
                }
            ),
            (
                mid,
                0.66 * duration,
                AnimationFloor {
                    pre_easing: Box::new(EaseInOut),
                    frames: frames - mid_frames
                }
            ),
            (src_end_rect, duration)
        ]
    } else {
        // the simpler case: choose some easing function as the pre-easing of an AnimationFloor
        // which operates on all frames, from the first to the last
        let easing = AnimationFloor {
            pre_easing: easing_function(ease_enum),
            frames,
        };
        keyframes![
            (src_rect_start, 0.0, easing),
            (src_end_rect, duration) // we don't need to specify a second easing function,
                                     // since this sequence won't be reversed, leading to
                                     // it never being used anyway
        ]
    }
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let ball = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Vec2::new(0.0, 0.0),
            60.0,
            1.0,
            Color::WHITE,
        )?;

        let img = graphics::Image::from_path(ctx, "/player_sheet.png")?;
        let s = MainState {
            ball,
            spritesheet: img,
            easing_enum: EasingEnum::Linear,
            animation_type: AnimationType::Idle,
            ball_animation: ball_sequence(&EasingEnum::Linear, 1.0),
            player_animation: player_sequence(&EasingEnum::Linear, &AnimationType::Idle, 1.0),
            duration: 1.0,
        };
        Ok(s)
    }
}

fn draw_info(canvas: &mut graphics::Canvas, info: String, position: Point2<f32>) {
    canvas.draw(
        graphics::Text::new(info).set_scale(40.),
        graphics::DrawParam::from(position).color(Color::WHITE),
    );
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let secs = ctx.time.delta().as_secs_f64();
        // advance the ball animation and reverse it once it reaches its end
        self.ball_animation.advance_and_maybe_reverse(secs);
        // advance the player animation and wrap around back to the beginning once it reaches its end
        self.player_animation.advance_and_maybe_wrap(secs);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::from([0.1, 0.2, 0.3, 1.0]));

        canvas.set_sampler(graphics::Sampler::nearest_clamp()); // because pixel art

        // draw some text showing the current parameters
        draw_info(
            &mut canvas,
            format!("Easing: {:?}", self.easing_enum),
            [300.0, 60.0].into(),
        );
        draw_info(
            &mut canvas,
            format!("Animation: {:?}", self.animation_type),
            [300.0, 110.0].into(),
        );
        draw_info(
            &mut canvas,
            format!("Duration: {:.2} s", self.duration),
            [300.0, 160.0].into(),
        );

        // draw the animated ball
        let ball_pos = self.ball_animation.now_strict().unwrap();
        canvas.draw(&self.ball, ball_pos);

        // draw the player
        let current_frame_src: graphics::Rect = self.player_animation.now_strict().unwrap().into();
        let scale = 3.0;
        canvas.draw(
            &self.spritesheet,
            graphics::DrawParam::new()
                .src(current_frame_src)
                .scale([scale, scale])
                .dest([470.0, 460.0])
                .offset([0.5, 1.0]),
        );

        canvas.finish(ctx)?;

        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        const DELTA: f32 = 0.2;
        match input.keycode {
            Some(KeyCode::Up | KeyCode::Down) => {
                // easing change
                let new_easing_enum = new_enum_after_key(
                    &self.easing_enum,
                    &EasingEnum::EaseInOut3Point,
                    KeyCode::Down,
                    KeyCode::Up,
                    input.keycode.unwrap(),
                );

                if self.easing_enum != new_easing_enum {
                    self.easing_enum = new_easing_enum;
                }
            }
            Some(KeyCode::Left | KeyCode::Right) => {
                // animation change
                let new_animation_type = new_enum_after_key(
                    &self.animation_type,
                    &AnimationType::Crawl,
                    KeyCode::Left,
                    KeyCode::Right,
                    input.keycode.unwrap(),
                );

                if self.animation_type != new_animation_type {
                    self.animation_type = new_animation_type;
                }
            }
            // duration change
            Some(KeyCode::W) => {
                self.duration += DELTA;
            }
            Some(KeyCode::S) => {
                if self.duration - DELTA > 0.1 {
                    self.duration -= DELTA;
                }
            }
            _ => {}
        }

        self.ball_animation = ball_sequence(&self.easing_enum, self.duration);
        self.player_animation =
            player_sequence(&self.easing_enum, &self.animation_type, self.duration);
        Ok(())
    }
}

fn new_enum_after_key<E: ToPrimitive + FromPrimitive>(
    old_enum: &E,
    max_enum: &E,
    dec_key: KeyCode,
    inc_key: KeyCode,
    key: KeyCode,
) -> E {
    let mut new_val = ToPrimitive::to_i32(old_enum).unwrap();
    new_val += match key {
        _ if key == dec_key => -1,
        _ if key == inc_key => 1,
        _ => 0,
    };

    let max_val = ToPrimitive::to_i32(max_enum).unwrap();
    if new_val < 0 {
        new_val = max_val;
    } else if new_val > max_val {
        new_val = 0;
    }

    FromPrimitive::from_i32(new_val).unwrap()
}

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("animation example", "ggez").add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;

    // instructions
    println!("CONTROLS:");
    println!("Left/Right: change animation");
    println!("Up/Down: change easing function");
    println!("W/S: change duration");

    event::run(ctx, event_loop, state)
}
