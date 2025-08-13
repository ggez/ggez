//! The simplest possible example that does something.
#![allow(clippy::unnecessary_wraps)]

use ggez::{
    conf::Conf,
    context::{ContextFields, Has, HasMut},
    event,
    filesystem::Filesystem,
    glam::*,
    graphics::{self, Color, GraphicsContext},
    input::{self, gamepad::GamepadContext, keyboard::KeyboardContext, mouse::MouseContext},
    timer::{self, TimeContext},
    GameResult,
};

struct MyContext {
    fs: Filesystem,
    gfx: GraphicsContext,
    keyboard: KeyboardContext,
    mouse: MouseContext,
    gamepad: GamepadContext,
    time: TimeContext,
    fields: ContextFields,
}

impl Has<Filesystem> for MyContext {
    #[inline]
    fn retrieve(&self) -> &Filesystem {
        &self.fs
    }
}

impl Has<GraphicsContext> for MyContext {
    #[inline]
    fn retrieve(&self) -> &GraphicsContext {
        &self.gfx
    }
}

impl HasMut<ContextFields> for MyContext {
    #[inline]
    fn retrieve_mut(&mut self) -> &mut ContextFields {
        &mut self.fields
    }
}

impl HasMut<GraphicsContext> for MyContext {
    #[inline]
    fn retrieve_mut(&mut self) -> &mut GraphicsContext {
        &mut self.gfx
    }
}

impl HasMut<timer::TimeContext> for MyContext {
    #[inline]
    fn retrieve_mut(&mut self) -> &mut timer::TimeContext {
        &mut self.time
    }
}

impl HasMut<input::keyboard::KeyboardContext> for MyContext {
    #[inline]
    fn retrieve_mut(&mut self) -> &mut input::keyboard::KeyboardContext {
        &mut self.keyboard
    }
}

impl HasMut<input::mouse::MouseContext> for MyContext {
    #[inline]
    fn retrieve_mut(&mut self) -> &mut input::mouse::MouseContext {
        &mut self.mouse
    }
}

impl HasMut<GamepadContext> for MyContext {
    #[inline]
    fn retrieve_mut(&mut self) -> &mut GamepadContext {
        &mut self.gamepad
    }
}

struct MainState {
    pos_x: f32,
    circle: graphics::Mesh,
}

impl MainState {
    fn new(ctx: &mut MyContext) -> GameResult<MainState> {
        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            vec2(0., 0.),
            100.0,
            2.0,
            Color::WHITE,
        )?;

        Ok(MainState { pos_x: 0.0, circle })
    }
}

impl event::EventHandler<MyContext> for MainState {
    fn update(&mut self, _ctx: &mut MyContext) -> GameResult {
        self.pos_x = self.pos_x % 800.0 + 1.0;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut MyContext) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

        canvas.draw(&self.circle, Vec2::new(self.pos_x, 380.0));

        canvas.finish(ctx)?;

        Ok(())
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("super_simple", "ggez");
    let (mut ctx, event_loop) =
        cb.custom_build::<MyContext>(|game_id: String, conf: Conf, fs: Filesystem| {
            let events_loop = winit::event_loop::EventLoop::new()?;
            let timer_context = timer::TimeContext::new();
            let graphics_context =
                graphics::context::GraphicsContext::new(&game_id, &events_loop, &conf, &fs)?;

            let ctx = MyContext {
                fs,
                gfx: graphics_context,
                time: timer_context,
                keyboard: input::keyboard::KeyboardContext::new(),
                mouse: input::mouse::MouseContext::new(),
                gamepad: GamepadContext::new()?,
                fields: ContextFields {
                    conf,
                    continuing: true,
                    quit_requested: false,
                },
            };

            Ok((ctx, events_loop))
        })?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
