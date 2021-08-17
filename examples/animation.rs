//! An example showcasing animation using `keyframe`.
//! Includes tweening and frame-by-frame animation.

use ggez::event;
use ggez::graphics::{self, Color, Rect, DrawParam, FilterMode, Text, TextFragment};
use ggez::mint::Point2;
use ggez::{Context, GameResult};
use glam::*;
use keyframe::{ease, functions::*, keyframes, AnimationSequence, EasingFunction};
use keyframe_derive::CanTween;
use std::env;
use std::path;
use ggez::input::keyboard::KeyCode;

struct MainState {
    spritesheet: graphics::Image,
    easing_enum: EasingEnum,
    animation_type: AnimationType,
    ball_animation: AnimationSequence<Point2<f32>>,
    player_animation: AnimationSequence<TweenableRect>,
    duration: f32
}

#[derive(Debug, Clone, PartialEq)]
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

fn ball_sequence(ease_enum: &EasingEnum, duration: f32) -> AnimationSequence<Point2<f32>> {
    let ball_pos_start: Point2<f32> = [120.0, 120.0].into();
    let ball_pos_end: Point2<f32> = [120.0, 420.0].into();

    if let EasingEnum::EaseInOut3Point = ease_enum {
        let mid_pos = ease(Linear, ball_pos_start, ball_pos_end, 0.33);
        keyframes![
            (ball_pos_start, 0.0, EaseInOut),
            (mid_pos, 0.66 * duration, EaseInOut), // reach about a third of the height two thirds of the duration
            (ball_pos_end, duration, EaseInOut)
        ]
    } else {
        match ease_enum {
            EasingEnum::Linear => keyframes![
                (ball_pos_start, 0.0, Linear),
                (ball_pos_end, duration, Linear)
            ],
            EasingEnum::EaseIn => keyframes![
                (ball_pos_start, 0.0, EaseIn),
                (ball_pos_end, duration, EaseIn)
            ],
            EasingEnum::EaseInOut => keyframes![
                (ball_pos_start, 0.0, EaseInOut),
                (ball_pos_end, duration, EaseInOut)
            ],
            EasingEnum::EaseOut => keyframes![
                (ball_pos_start, 0.0, EaseOut),
                (ball_pos_end, duration, EaseOut)
            ],
            EasingEnum::EaseInCubic => keyframes![
                (ball_pos_start, 0.0, EaseInCubic),
                (ball_pos_end, duration, EaseInCubic)
            ],
            EasingEnum::EaseOutCubic => keyframes![
                (ball_pos_start, 0.0, EaseOutCubic),
                (ball_pos_end, duration, EaseOutCubic)
            ],
            EasingEnum::EaseInOutCubic => keyframes![
                (ball_pos_start, 0.0, EaseInOutCubic),
                (ball_pos_end, duration, EaseInOutCubic)
            ],
            EasingEnum::Bezier =>
                {
                    let bezier_function = BezierCurve::from([0.6, 0.04].into(), [0.98, 0.335].into());
                    keyframes![
                        (ball_pos_start, 0.0, bezier_function),
                        (ball_pos_end, duration, bezier_function)
                    ]
                },
            _ => panic!()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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
fn src_y(anim_type: &AnimationType) -> f32
{
    let row = match anim_type
    {
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
fn src_x_end(anim_type: &AnimationType) -> f32
{
    (frame_count(anim_type) - 1) as f32 / FRAME_COLUMNS as f32
}

fn frame_count(anim_type: &AnimationType) -> i32
{
    match anim_type
    {
        AnimationType::Idle => 7,
        AnimationType::Run => 8,
        AnimationType::FrontFlip => 14,
        AnimationType::Roll => 10,
        AnimationType::Crawl => 8,
    }
}

#[derive(CanTween, Clone, Copy)]
struct TweenableRect
{
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl From<TweenableRect> for Rect
{
    fn from(t_rect: TweenableRect) -> Self {
        Rect {
            x: t_rect.x,
            y: t_rect.y,
            w: t_rect.w,
            h: t_rect.h
        }
    }
}

struct AnimationFloor<E: EasingFunction>
{
    pre_easing: E,
    frames: i32
}
impl<E: EasingFunction> EasingFunction for AnimationFloor<E> {
    #[inline]
    fn y(&self, x: f64) -> f64 { (self.pre_easing.y(x) * (self.frames) as f64).floor() / (self.frames-1) as f64 }
}

fn player_sequence(ease_enum: &EasingEnum, anim_type: &AnimationType, duration: f32) -> AnimationSequence<TweenableRect> {
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
    let src_rect_start = TweenableRect{ x: src_x_start, y: src_y, w, h };
    let src_end_rect = TweenableRect{ x: src_x_end, y: src_y, w, h };

    let frames = frame_count(anim_type);

    if let EasingEnum::EaseInOut3Point = ease_enum {
        let mid = ease(AnimationFloor { pre_easing: Linear, frames }, src_rect_start, src_end_rect, 0.33);
        let mid_frames = (frames as f32 * 0.33).floor() as i32;
        // we need to adapt the frame count for each keyframe
        // only the frames that are to be played until the next keyframe count
        keyframes![
            (src_rect_start, 0.0, AnimationFloor { pre_easing: EaseInOut, frames: mid_frames + 1 }),
            (mid, 0.66 * duration, AnimationFloor { pre_easing: EaseInOut, frames: frames - mid_frames }),
            (src_end_rect, duration)
        ]
    } else {
        match ease_enum {
            EasingEnum::Linear => {
                keyframes![
                    (src_rect_start, 0.0, AnimationFloor { pre_easing: Linear, frames }),
                    (src_end_rect, duration)    // we don't need to specify a second easing function,
                                                // since this sequence won't be reversed, leading to
                                                // it never being used anyway
                ]
            },
            EasingEnum::EaseIn =>  {
                keyframes![
                    (src_rect_start, 0.0, AnimationFloor { pre_easing: EaseIn, frames }),
                    (src_end_rect, duration)
                ]
            },
            EasingEnum::EaseInOut =>  {
                keyframes![
                    (src_rect_start, 0.0, AnimationFloor { pre_easing: EaseInOut, frames }),
                    (src_end_rect, duration)
                ]
            },
            EasingEnum::EaseOut =>  {
                keyframes![
                    (src_rect_start, 0.0, AnimationFloor { pre_easing: EaseOut, frames }),
                    (src_end_rect, duration)
                ]
            },
            EasingEnum::EaseInCubic =>  {
                keyframes![
                    (src_rect_start, 0.0, AnimationFloor { pre_easing: EaseInCubic, frames }),
                    (src_end_rect, duration)
                ]
            },
            EasingEnum::EaseOutCubic =>  {
                keyframes![
                    (src_rect_start, 0.0, AnimationFloor { pre_easing: EaseOutCubic, frames }),
                    (src_end_rect, duration)
                ]
            },
            EasingEnum::EaseInOutCubic =>  {
                keyframes![
                    (src_rect_start, 0.0, AnimationFloor { pre_easing: EaseInOutCubic, frames }),
                    (src_end_rect, duration)
                ]
            },
            EasingEnum::Bezier =>
                {
                    let bezier_function = BezierCurve::from([0.6, 0.04].into(), [0.98, 0.335].into());
                    keyframes![
                        (src_rect_start, 0.0, AnimationFloor { pre_easing: bezier_function, frames }),
                        (src_end_rect, duration)
                    ]
                },
            _ => panic!()
        }
    }
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let mut img = graphics::Image::new(ctx, "/player_sheet.png")?;
        img.set_filter(FilterMode::Nearest);
        let s = MainState {
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

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // advance the ball animation and reverse it once it reaches its end
        self.ball_animation.advance_and_maybe_reverse(ggez::timer::delta(ctx).as_secs_f64());
        // advance the player animation and wrap around back to the beginning once it reaches its end
        self.player_animation.advance_and_maybe_wrap(ggez::timer::delta(ctx).as_secs_f64());
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        // draw some text showing the current parameters
        let t = Text::new(TextFragment {
            text: format!("Easing: {:?}", self.easing_enum),
            font: None,
            scale: Some(ggez::graphics::PxScale::from(40.0)),
            ..Default::default()
        });
        graphics::draw(ctx, &t, DrawParam::default()
            .dest([300.0, 60.0])
            .color(Color::WHITE)
        )?;

        let t = Text::new(TextFragment {
            text: format!("Animation: {:?}", self.animation_type),
            font: None,
            scale: Some(ggez::graphics::PxScale::from(40.0)),
            ..Default::default()
        });
        graphics::draw(ctx, &t, DrawParam::default()
            .dest([300.0, 110.0])
            .color(Color::WHITE)
        )?;

        let t = Text::new(TextFragment {
            text: format!("Duration: {:.2} s", self.duration),
            font: None,
            scale: Some(ggez::graphics::PxScale::from(40.0)),
            ..Default::default()
        });
        graphics::draw(ctx, &t, DrawParam::default()
            .dest([300.0, 160.0])
            .color(Color::WHITE)
        )?;

        // draw the animated ball
        let ball = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Vec2::new(0.0, 0.0),
            60.0,
            1.0,
            Color::WHITE,
        )?;
        let ball_pos = self.ball_animation.now_strict().unwrap();
        graphics::draw(ctx, &ball, (ball_pos,))?;

        // draw the player
        let current_frame_src: Rect = self.player_animation.now_strict().unwrap().into();
        let scale = 3.0;
        let draw_p = DrawParam::default()
            .src(current_frame_src)
            .scale(Vec2::new(scale, scale))
            .dest([470.0, 460.0])
            .offset([0.5, 1.0]);
        graphics::draw(ctx, &self.spritesheet, draw_p)?;

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: ggez::input::keyboard::KeyCode,
        _keymods: ggez::input::keyboard::KeyMods,
        _repeat: bool,
    ) {
        const DELTA: f32 = 0.2;
        match keycode
        {
            KeyCode::Up | KeyCode::Down => {
                // easing change
                let old_val: i32 = unsafe { std::mem::transmute(self.easing_enum.clone()) };
                let max_val: i32 = unsafe { std::mem::transmute(EasingEnum::EaseInOut3Point) };
                let new_val = new_enum_val_after_key(old_val, max_val, &KeyCode::Down, &KeyCode::Up, &keycode);
                let new_easing_enum = unsafe { std::mem::transmute::<i32, EasingEnum>(new_val) };

                if self.easing_enum != new_easing_enum {
                self.easing_enum = new_easing_enum;
                }
            }
            KeyCode::Left | KeyCode::Right => {
                // animation change
                let old_val: i32 = unsafe { std::mem::transmute(self.animation_type.clone()) };
                let max_val: i32 = unsafe { std::mem::transmute(AnimationType::Crawl) };
                let new_val = new_enum_val_after_key(old_val, max_val, &KeyCode::Left, &KeyCode::Right, &keycode);
                let new_animation_type = unsafe { std::mem::transmute::<i32, AnimationType>(new_val) };

                if self.animation_type != new_animation_type {
                    self.animation_type = new_animation_type;
                }
            }
            // duration change
            KeyCode::W => {
                self.duration += DELTA;
            }
            KeyCode::S => {
                if self.duration - DELTA > 0.1
                {
                    self.duration -= DELTA;
                }
            }
            _ => {}
        }

        self.ball_animation = ball_sequence(&self.easing_enum, self.duration);
        self.player_animation = player_sequence(&self.easing_enum, &self.animation_type, self.duration);
    }
}

// this could probably be done more elegantly using the num_derive and num_traits crates, but I'm unsure as to whether I really want to
fn new_enum_val_after_key(old_val: i32, max_val: i32, dec_key: &KeyCode, inc_key: &KeyCode, key: &KeyCode) -> i32
{
    let mut new_val = old_val;
    new_val += match key {
        _ if *key == *dec_key => -1,
        _ if *key == *inc_key => 1,
        _ => 0,
    };

    if new_val < 0 {
        new_val = max_val;
    } else if new_val
        > max_val
    {
        new_val = 0;
    }

    new_val
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
