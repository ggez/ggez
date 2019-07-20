//! How to draw a 3D cube in ggez.
//!
//! ggez doesn't provide any 3D drawing itself, but it exposes
//! the underlying `gfx-rs` data types, so you can bypass ggez's
//! drawing code entirely and write your own.

#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate ggez;
extern crate nalgebra;

use gfx::texture;
use gfx::traits::FactoryExt;
use gfx::Factory;

use ggez::event;
use ggez::graphics;
use ggez::{Context, GameResult};
use nalgebra as na;
use std::env;
use std::f32;
use std::path;

type Isometry3 = na::Isometry3<f32>;
type Point3 = na::Point3<f32>;
type Vector3 = na::Vector3<f32>;
// ColorFormat and DepthFormat are hardwired into ggez's drawing code,
// and there isn't a way to easily change them, so for the moment we just have
// to know what they are and use the same settings.
type ColorFormat = gfx::format::Srgba8;
type DepthFormat = gfx::format::DepthStencil;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 4] = "a_Pos",
        tex_coord: [f32; 2] = "a_TexCoord",
    }

    constant Locals {
        transform: [[f32; 4]; 4] = "u_Transform",
        rotation: [[f32; 4]; 4] = "u_Rotation",
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
            pos: [f32::from(p[0]), f32::from(p[1]), f32::from(p[2]), 1.0],
            tex_coord: [f32::from(t[0]), f32::from(t[1])],
        }
    }
}

fn default_view() -> Isometry3 {
    // Eye location, target location, up-vector
    Isometry3::look_at_rh(
        &Point3::new(1.5f32, -5.0, 3.0),
        &Point3::new(0f32, 0.0, 0.0),
        &Vector3::z_axis(),
    )
}

struct MainState {
    frames: usize,
    rotation: f32,

    // All the gfx-rs state stuff we need to keep track of.
    data: pipe::Data<gfx_device_gl::Resources>,
    pso: gfx::PipelineState<gfx_device_gl::Resources, pipe::Meta>,
    slice: gfx::Slice<gfx_device_gl::Resources>,
}

impl MainState {
    fn new(ctx: &mut Context) -> Self {
        let (factory, _device, _encoder, depth_view, color_view) = graphics::gfx_objects(ctx);

        // Shaders.
        let vs = br#"#version 150 core

in vec4 a_Pos;
in vec2 a_TexCoord;
out vec2 v_TexCoord;

uniform Locals {
    mat4 u_Transform;
    mat4 u_Rotation;
};

void main() {
    v_TexCoord = a_TexCoord;
    gl_Position = u_Transform * u_Rotation * a_Pos ;
    gl_ClipDistance[0] = 1.0;
}"#;
        let fs = br#"#version 150 core

in vec2 v_TexCoord;
out vec4 Target0;
uniform sampler2D t_Color;

void main() {
    vec4 tex = texture(t_Color, v_TexCoord);
    float blend = dot(v_TexCoord-vec2(0.5,0.5), v_TexCoord-vec2(0.5,0.5));
    Target0 = mix(tex, vec4(0.0,0.0,0.0,0.0), blend*1.0);
}"#;

        // Cube geometry
        let vertex_data = [
            // top (0, 0, 1)
            Vertex::new([-1, -1, 1], [0, 0]),
            Vertex::new([1, -1, 1], [1, 0]),
            Vertex::new([1, 1, 1], [1, 1]),
            Vertex::new([-1, 1, 1], [0, 1]),
            // bottom (0, 0, -1)
            Vertex::new([-1, 1, -1], [1, 0]),
            Vertex::new([1, 1, -1], [0, 0]),
            Vertex::new([1, -1, -1], [0, 1]),
            Vertex::new([-1, -1, -1], [1, 1]),
            // right (1, 0, 0)
            Vertex::new([1, -1, -1], [0, 0]),
            Vertex::new([1, 1, -1], [1, 0]),
            Vertex::new([1, 1, 1], [1, 1]),
            Vertex::new([1, -1, 1], [0, 1]),
            // left (-1, 0, 0)
            Vertex::new([-1, -1, 1], [1, 0]),
            Vertex::new([-1, 1, 1], [0, 0]),
            Vertex::new([-1, 1, -1], [0, 1]),
            Vertex::new([-1, -1, -1], [1, 1]),
            // front (0, 1, 0)
            Vertex::new([1, 1, -1], [1, 0]),
            Vertex::new([-1, 1, -1], [0, 0]),
            Vertex::new([-1, 1, 1], [0, 1]),
            Vertex::new([1, 1, 1], [1, 1]),
            // back (0, -1, 0)
            Vertex::new([1, -1, 1], [0, 0]),
            Vertex::new([-1, -1, 1], [1, 0]),
            Vertex::new([-1, -1, -1], [1, 1]),
            Vertex::new([1, -1, -1], [0, 1]),
        ];

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let index_data: &[u16] = &[
             0,  1,  2,  2,  3,  0, // top
             4,  5,  6,  6,  7,  4, // bottom
             8,  9, 10, 10, 11,  8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ];

        // Create vertex buffer
        let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, index_data);

        // Create 1-pixel blue texture.
        let texels = [[0x20, 0xA0, 0xC0, 0x00]];
        let (_, texture_view) = factory
            .create_texture_immutable::<gfx::format::Rgba8>(
                texture::Kind::D2(1, 1, texture::AaMode::Single),
                texture::Mipmap::Provided,
                &[&texels],
            )
            .unwrap();

        let sinfo =
            texture::SamplerInfo::new(texture::FilterMethod::Bilinear, texture::WrapMode::Clamp);

        // Create pipeline state object
        let pso = factory.create_pipeline_simple(vs, fs, pipe::new()).unwrap();

        // Aspect ratio, FOV, znear, zfar
        let proj = na::Perspective3::new(4.0 / 3.0, f32::consts::PI / 4.0, 1.0, 10.0);
        let transform = proj.as_matrix() * default_view().to_homogeneous();

        // Bundle all the data together.
        let data = pipe::Data {
            vbuf,
            transform: transform.into(),
            locals: factory.create_constant_buffer(1),
            color: (texture_view, factory.create_sampler(sinfo)),
            // We use the (undocumented-but-useful) gfx::memory::Typed here
            // to convert ggez's raw render and depth buffers into ones with
            // compile-time type information.
            out_color: gfx::memory::Typed::new(color_view),
            out_depth: gfx::memory::Typed::new(depth_view),
        };

        MainState {
            frames: 0,
            data,
            pso,
            slice,
            rotation: 0.0,
        }
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.rotation += 0.01;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // Do gfx-rs drawing
        {
            let (_factory, device, encoder, _depthview, _colorview) = graphics::gfx_objects(ctx);
            encoder.clear(&self.data.out_color, [0.1, 0.1, 0.1, 1.0]);

            let rotation = na::Matrix4::from_scaled_axis(na::Vector3::z() * self.rotation);

            let locals = Locals {
                transform: self.data.transform,
                rotation: rotation.into(),
            };
            encoder.update_constant_buffer(&self.data.locals, &locals);
            encoder.clear_depth(&self.data.out_depth, 1.0);

            encoder.draw(&self.slice, &self.pso, &self.data);
            encoder.flush(device);
        }

        // Do ggez drawing
        let dest_point1 = na::Point2::new(10.0, 210.0);
        let dest_point2 = na::Point2::new(10.0, 250.0);
        // graphics::draw(ctx, &self.text1, (dest_point1,))?;
        // graphics::draw(ctx, &self.text2, (dest_point2,))?;

        graphics::queue_text(
            ctx,
            &graphics::Text::new("You can mix ggez and gfx drawing;"),
            dest_point1,
            None,
        );
        graphics::queue_text(
            ctx,
            &graphics::Text::new("it basically draws gfx stuff first, then ggez"),
            dest_point2,
            None,
        );
        graphics::draw_queued_text(
            ctx,
            graphics::DrawParam::default(),
            None,
            graphics::FilterMode::Linear,
        )?;
        graphics::present(ctx)?;
        self.frames += 1;
        if (self.frames % 10) == 0 {
            println!("FPS: {}", ggez::timer::fps(ctx));
        }
        Ok(())
    }
}

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("cube", "ggez").add_resource_path(resource_dir);

    let (ctx, events_loop) = &mut cb.build()?;
    let state = &mut MainState::new(ctx);
    event::run(ctx, events_loop, state)
}
