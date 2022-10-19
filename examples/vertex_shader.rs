//! An example demonstrating vertex shaders.

use ggez::event;
use ggez::glam::*;
use ggez::graphics::AsStd140;
use ggez::graphics::{self, Color, DrawParam};
use ggez::{Context, GameResult};
use mint::ColumnMatrix4;

#[derive(AsStd140)]
struct ShaderUniforms {
    transform: ColumnMatrix4<f32>,
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
        let shader = graphics::Shader::new_wgsl(
            &ctx.gfx,
            include_str!("../resources/vertex.wgsl"),
            "fs_main",
        )
        .with_vertex("vs_main");
        let shader_params = graphics::ShaderParams::new(
            &mut ctx.gfx,
            &ShaderUniforms {
                transform: Mat4::IDENTITY.into(),
                rotation: Mat4::IDENTITY.into(),
            },
            &[],
            &[],
        );

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

        canvas.set_shader(self.shader.clone());
        self.shader_params.set_uniforms(
            &mut ctx.gfx,
            &ShaderUniforms {
                transform: Mat4::from_translation(Vec3::new(200.0, 100.0, 0.0)).into(),
                rotation: Mat4::from_rotation_z(ctx.time.time_since_start().as_secs_f32()).into(),
            },
        );
        canvas.draw(&self.square_mesh, DrawParam::default());

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

    let cb = ggez::ContextBuilder::new("colorspace", "ggez").add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
