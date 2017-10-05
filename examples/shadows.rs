#[macro_use]
extern crate gfx;
extern crate ggez;

use ggez::*;
use ggez::event::*;
use ggez::graphics::*;
use std::env;
use std::path;

gfx_defines!{
    constant Light {
        color: [f32; 4] = "u_Color",
        pos: [f32; 2] = "u_Pos",
        screen_size: [f32; 2] = "u_ScreenSize",
        glow: f32 = "u_Glow",
        strength: f32 = "u_Strength",
    }
}

const OCCLUSIONS_SHADER_SOURCE: &[u8] = b"#version 150 core

uniform sampler2D t_Texture;
in vec2 v_Uv;
out vec4 Target0;

layout (std140) uniform Light {
    vec4 u_Color;
    vec2 u_Pos;
    vec2 u_ScreenSize;
    float u_Glow;
    float u_Strength;
};

void main() {
    float dist = 1.0;
    float theta = radians(v_Uv.x * 360.0);
    vec2 dir = vec2(cos(theta), sin(theta));
    vec2 pos = u_Pos / u_ScreenSize;
    for(int i = 0; i < 1024; i++) {
        float fi = i;
        float r = fi / 1024.0;
        vec2 rel = r * dir;
        vec2 p = clamp(pos+rel, 0.0, 1.0);
        if (texture(t_Texture, p).a == 1.0) {
            dist = distance(pos, p) * 0.5;
            break;
        }
    }

    float others = dist == 1.0 ? 0.0 : dist;
    Target0 = vec4(dist, others, others, 1.0);
}
";

const SHADOWS_SHADER_SOURCE: &[u8] = b"#version 150 core

uniform sampler2D t_Texture;
in vec2 v_Uv;
out vec4 Target0;

layout (std140) uniform Light {
    vec4 u_Color;
    vec2 u_Pos;
    vec2 u_ScreenSize;
    float u_Glow;
    float u_Strength;
};

void main() {
    vec2 coord = gl_FragCoord.xy / u_ScreenSize;
    vec2 pos = u_Pos / u_ScreenSize;

    vec2 rel = coord - pos;
    float theta = atan(rel.y, rel.x);
    float ox = degrees(theta) / 360.0;
    if (ox < 0) {
        ox += 1.0;
    }
    float r = length(rel);
    float occl = texture(t_Texture, vec2(ox, 0.5)).r * 2.0;

    float intencity = 1.0;
    if (r < occl) {
        float p = u_Strength + u_Glow;
        float d = distance(gl_FragCoord.xy, u_Pos);
        intencity = 1.0 - clamp(p/(d*d), 0.0, 1.0);
    }

    Target0 = vec4(0.0, 0.0, 0.0, intencity);
}
";

struct MainState {
    background: Image,
    tile: Image,
    text: Text,
    light: Light,
    foreground: Canvas,
    occlusions: Canvas,
    occlusions_shader: PixelShader<Light>,
    shadows_shader: PixelShader<Light>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let background = Image::new(ctx, "/background.png")?;
        let tile = Image::new(ctx, "/tile.png")?;
        let text = {
            let font = Font::new(ctx, "/DejaVuSerif.ttf", 48)?;
            Text::new(ctx, "SHADOWS...", &font)?
        };
        let screen_size = {
            let coords = graphics::get_screen_coordinates(ctx);
            [coords.w, coords.h.abs()]
        };
        let light = Light {
            pos: [0.0, 0.0],
            color: [0.7, 0.64, 0.34, 1.0],
            screen_size,
            glow: 0.0,
            strength: 2500.0,
        };
        let foreground = Canvas::with_window_size(ctx)?;
        let occlusions = Canvas::new(ctx, 360, 1)?;
        let occlusions_shader =
            PixelShader::from_u8(ctx, OCCLUSIONS_SHADER_SOURCE, light, "Light")?;
        let shadows_shader = PixelShader::from_u8(ctx, SHADOWS_SHADER_SOURCE, light, "Light")?;

        Ok(MainState {
            background,
            tile,
            text,
            light,
            foreground,
            occlusions,
            occlusions_shader,
            shadows_shader,
        })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.light.glow = 200.0 * ((timer::get_ticks(ctx) as f32) / 20.0).cos();
        self.occlusions_shader.send(ctx, self.light)?;
        self.shadows_shader.send(ctx, self.light)?;

        if timer::get_ticks(ctx) % 100 == 0 {
            println!("Delta frame time: {:?} ", timer::get_delta(ctx));
            println!("Average FPS: {}", timer::get_fps(ctx));
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let center = DrawParam {
            dest: Point2::new(
                self.light.screen_size[0] / 2.0,
                self.light.screen_size[1] / 2.0,
            ),
            ..Default::default()
        };

        graphics::set_canvas(ctx, Some(&self.foreground));
        graphics::set_background_color(ctx, [0.0; 4].into());
        graphics::draw_ex(
            ctx,
            &self.tile,
            DrawParam {
                dest: Point2::new(598.0, 124.0),
                ..Default::default()
            },
        )?;
        graphics::draw_ex(
            ctx,
            &self.tile,
            DrawParam {
                dest: Point2::new(92.0, 350.0),
                ..Default::default()
            },
        )?;
        graphics::draw_ex(
            ctx,
            &self.tile,
            DrawParam {
                dest: Point2::new(442.0, 468.0),
                rotation: 0.5,
                ..Default::default()
            },
        )?;
        graphics::draw_ex(ctx, &self.text, center)?;

        {
            let _shader_lock = graphics::use_shader(ctx, &self.occlusions_shader);
            graphics::set_canvas(ctx, Some(&self.occlusions));
            graphics::draw_ex(ctx, &self.foreground, center)?;
        }

        graphics::set_canvas(ctx, None);
        graphics::set_color(ctx, self.light.color.into())?;
        graphics::clear(ctx);
        graphics::draw_ex(ctx, &self.background, center)?;
        {
            let _shader_lock = graphics::use_shader(ctx, &self.shadows_shader);
            let param = DrawParam {
                scale: Point2::new(self.light.screen_size[0] / 360.0, self.light.screen_size[1]),
                ..center
            };
            graphics::draw_ex(ctx, &self.occlusions, param)?;
        }
        graphics::draw_ex(ctx, &self.foreground, center)?;

        // Uncomment following two lines to visualize the 1D occlusions canvas
        // graphics::set_color(ctx, [1.0; 4].into())?;
        // graphics::draw_ex(ctx, &self.occlusions, center)?;

        graphics::present(ctx);
        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        ctx: &mut Context,
        _state: MouseState,
        x: i32,
        y: i32,
        _xrel: i32,
        _yrel: i32,
    ) {
        let pos = {
            let pos = Point2::new(x as f32, y as f32);
            graphics::convert_screen_coordinates(ctx, pos)
        };
        self.light.pos = [pos.x, pos.y];
    }
}

pub fn main() {
    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("shadows", "ggez", c).unwrap();

    // We add the CARGO_MANIFEST_DIR/resources do the filesystems paths so
    // we we look in the cargo project for files.
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        ctx.filesystem.mount(&path, true);
    }

    let state = &mut MainState::new(ctx).unwrap();
    event::run(ctx, state).unwrap();
}
