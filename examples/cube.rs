extern crate ggez;
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate cgmath;


use cgmath::{Deg, Matrix4, Point3, Vector3};
use gfx::texture;
use gfx::traits::Device;
use gfx::traits::FactoryExt;
use gfx::Factory;

use ggez::conf;
use ggez::event;
use ggez::{GameResult, Context};
use ggez::graphics;
use std::time::Duration;


type ColorFormat = gfx::format::Srgba8;
type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 4] = "a_Pos",
        tex_coord: [f32; 2] = "a_TexCoord",
    }

    constant Locals {
        transform: [[f32; 4]; 4] = "u_Transform",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        transform: gfx::Global<[[f32; 4]; 4]> = "u_Transform",
        locals: gfx::ConstantBuffer<Locals> = "Locals",
        color: gfx::TextureSampler<[f32; 4]> = "t_Color",
        out_color: gfx::RenderTarget<ColorFormat> = "Target0",
        out_depth: gfx::DepthTarget<DepthFormat> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}


impl Vertex {
    fn new(p: [i8; 3], t: [i8; 2]) -> Vertex {
        Vertex {
            pos: [p[0] as f32, p[1] as f32, p[2] as f32, 1.0],
            tex_coord: [t[0] as f32, t[1] as f32],
        }
    }
}

fn default_view() -> Matrix4<f32> {
    Matrix4::look_at(
        Point3::new(1.5f32, -5.0, 3.0),
        Point3::new(0f32, 0.0, 0.0),
        Vector3::unit_z(),
    )
}

struct MainState {
    text: graphics::Text,
    frames: usize,
    data: pipe::Data<gfx_device_gl::Resources>,
    pso: gfx::PipelineState<gfx_device_gl::Resources, pipe::Meta>,
    encoder: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
    slice: gfx::Slice<gfx_device_gl::Resources>,
}

impl MainState {
    fn new(ctx: &mut Context) -> Self {

        let font = graphics::Font::new(ctx, "/DejaVuSerif.ttf", 48).unwrap();
        let text = graphics::Text::new(ctx, "Hello world!", &font).unwrap();

        let gfx = &mut ctx.gfx_context;
        let color_view = gfx.get_color_view();
        let depth_view = gfx.get_depth_view();
        let factory = gfx.get_factory();

        let vs = r#"#version 150 core

in vec4 a_Pos;
in vec2 a_TexCoord;
out vec2 v_TexCoord;

uniform Locals {
    mat4 u_Transform;
};

void main() {
    v_TexCoord = a_TexCoord;
    gl_Position = u_Transform * a_Pos;
    gl_ClipDistance[0] = 1.0;
}"#.as_bytes();
        let fs = r#"#version 150 core

in vec2 v_TexCoord;
out vec4 Target0;
uniform sampler2D t_Color;

void main() {
    vec4 tex = texture(t_Color, v_TexCoord);
    float blend = dot(v_TexCoord-vec2(0.5,0.5), v_TexCoord-vec2(0.5,0.5));
    Target0 = mix(tex, vec4(0.0,0.0,0.0,0.0), blend*1.0);
}"#.as_bytes();

        let vertex_data = [
            // top (0, 0, 1)
            Vertex::new([-100, -100,  100], [0, 0]),
            Vertex::new([ 100, -100,  100], [1, 0]),
            Vertex::new([ 100,  100,  100], [1, 1]),
            Vertex::new([-100,  100,  100], [0, 1]),
            // bottom (0, 0, -1)
            Vertex::new([-100,  100, -100], [1, 0]),
            Vertex::new([ 100,  100, -100], [0, 0]),
            Vertex::new([ 100, -100, -100], [0, 1]),
            Vertex::new([-100, -100, -100], [1, 1]),
            // right (1, 0, 0)
            Vertex::new([ 100, -100, -100], [0, 0]),
            Vertex::new([ 100,  100, -100], [1, 0]),
            Vertex::new([ 100,  100,  100], [1, 1]),
            Vertex::new([ 100, -100,  100], [0, 1]),
            // left (-1, 0, 0)
            Vertex::new([-100, -100,  100], [1, 0]),
            Vertex::new([-100,  100,  100], [0, 0]),
            Vertex::new([-100,  100, -100], [0, 1]),
            Vertex::new([-100, -100, -100], [1, 1]),
            // front (0, 1, 0)
            Vertex::new([ 100,  100, -100], [1, 0]),
            Vertex::new([-100,  100, -100], [0, 0]),
            Vertex::new([-100,  100,  100], [0, 1]),
            Vertex::new([ 100,  100,  100], [1, 1]),
            // back (0, -1, 0)
            Vertex::new([ 100, -100,  100], [0, 0]),
            Vertex::new([-100, -100,  100], [1, 0]),
            Vertex::new([-100, -100, -100], [1, 1]),
            Vertex::new([ 100, -100, -100], [0, 1]),
        ];

        let index_data: &[u16] = &[
             0,  1,  2,  2,  3,  0, // top
             4,  5,  6,  6,  7,  4, // bottom
             8,  9, 10, 10, 11,  8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ];

        let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, index_data);

        let texels = [[0x20, 0xA0, 0xC0, 0x00]];
        let (_, texture_view) = factory.create_texture_immutable::<gfx::format::Rgba8>(
            texture::Kind::D2(1, 1, texture::AaMode::Single), &[&texels]
            ).unwrap();

        let sinfo = texture::SamplerInfo::new(
            texture::FilterMethod::Bilinear,
            texture::WrapMode::Clamp);

        let pso = factory.create_pipeline_simple(
            vs,
            fs,
            pipe::new()
        ).unwrap();

        let proj = cgmath::perspective(Deg(45.0f32), 4.0/3.0, 1.0, 10.0);

        let data = pipe::Data {
            vbuf: vbuf,
            transform: (proj * default_view()).into(),
            locals: factory.create_constant_buffer(1),
            color: (texture_view, factory.create_sampler(sinfo)),
            out_color: color_view,
            out_depth: depth_view,
        };

        let encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

        let s = MainState {
            text: text,
            frames: 0,

            data: data,
            pso: pso,
            encoder: encoder,
            slice: slice,
        };
        s
    }
}


impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context, _dt: Duration) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let dest_point = graphics::Point::new(self.text.width() as f32 / 2.0 + 10.0,
                                              self.text.height() as f32 / 2.0 + 10.0);
        self.encoder.clear(&self.data.out_color, [0.3, 0.2, 0.1, 1.0].into());
        self.encoder.draw(&self.slice, &self.pso, &self.data);
        self.encoder.flush(ctx.gfx_context.get_device());


        let dest_point = graphics::Point::new(self.text.width() as f32 / 2.0 + 10.0,
                                              self.text.height() as f32 / 2.0 + 10.0);
        graphics::draw(ctx, &self.text, dest_point, 0.0)?;
        graphics::present(ctx);
        self.frames += 1;
        if (self.frames % 100) == 0 {
            println!("FPS: {}", ggez::timer::get_fps(ctx));
        }
        Ok(())
    }
}

pub fn main() {
    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("helloworld", "ggez", c).unwrap();
    let state = &mut MainState::new(ctx);
    if let Err(e) = event::run(ctx, state) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
