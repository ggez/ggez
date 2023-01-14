//! An example demonstrating vertex shaders.

use crevice::std140::AsStd140;
use ggez::event;
use ggez::glam::*;
use ggez::graphics::{self, Color, DrawParam};
use ggez::{Context, GameResult};
use mint::ColumnMatrix4;

#[derive(AsStd140)]
struct ShaderUniforms {
    rotation: ColumnMatrix4<f32>,
}

struct MainState {
    square_mesh: graphics::Mesh,
    shader: graphics::Shader,
    shader_params: graphics::ShaderParams<ShaderUniforms>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let square_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(0.0, 0.0, 400.0, 400.0),
            Color::WHITE,
        )?;
        let shader = graphics::ShaderBuilder::new_wgsl()
            .vertex_path("/vertex.wgsl")
            .build(ctx)?;
        let shader_params = graphics::ShaderParamsBuilder::new(&ShaderUniforms {
            rotation: Mat4::IDENTITY.into(),
        })
        .build(ctx);

        let s = MainState {
            square_mesh,
            shader,
            shader_params,
        };
        Ok(s)
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);

        self.shader_params.set_uniforms(
            ctx,
            &ShaderUniforms {
                rotation: Mat4::from_rotation_z(ctx.time.time_since_start().as_secs_f32()).into(),
            },
        );
        canvas.set_shader(&self.shader);
        canvas.set_shader_params(&self.shader_params);
        canvas.draw(
            &self.square_mesh,
            DrawParam::default().dest(Vec2::new(200.0, 100.0)),
        );

        canvas.finish(ctx)?;

        Ok(())
    }
}

pub fn main() -> GameResult {
    use std::env;
    use std::path;
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("vertex_shader", "ggez").add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
