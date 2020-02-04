//! The `graphics` module performs the actual drawing of images, text, and other
//! objects with the [`Drawable`](trait.Drawable.html) trait.  It also handles
//! basic loading of images and text.
//!
//! This module also manages graphics state, coordinate systems, etc.
//! The default coordinate system has the origin in the upper-left
//! corner of the screen, with Y increasing downwards.
//!
//! This library differs significantly in performance characteristics from the
//! `LÖVE` library that it is based on. Many operations that are batched by default
//! in love (e.g. drawing primitives like rectangles or circles) are *not* batched
//! in `ggez`, so render loops with a large number of draw calls can be very slow.
//! The primary solution to efficiently rendering a large number of primitives is
//! a [`SpriteBatch`](spritebatch/struct.SpriteBatch.html), which can be orders
//! of magnitude more efficient than individual
//! draw calls.

// use std::collections::HashMap;
use std::convert::From;
use std::rc::Rc;
// use std::fmt;
// use std::path::Path;
// use std::u16;

use ggraphics as gg;
use glutin;

use crate::conf;
use crate::conf::WindowMode;
use crate::context::Context;
//use crate::context::DebugId;
// use crate::GameError;
use crate::GameResult;

pub(crate) mod drawparam;
pub(crate) mod image;
//pub(crate) mod mesh;
pub(crate) mod text;
pub(crate) mod types;

pub use crate::graphics::drawparam::*;
pub use crate::graphics::image::*;
//pub use crate::graphics::mesh::*;
pub use crate::graphics::text::*;
pub use crate::graphics::types::*;

pub type WindowCtx = glutin::WindowedContext<glutin::PossiblyCurrent>;

pub type SamplerSpec = gg::SamplerSpec;

#[derive(Debug)]
pub struct GraphicsContext {
    // TODO: OMG these names
    pub glc: Rc<gg::GlContext>,
    pub(crate) win: WindowCtx,
    pub(crate) screen_pass: RenderPass,
    pub(crate) passes: Vec<gg::RenderPass>,
}

impl GraphicsContext {
    pub(crate) fn new(gl: gg::glow::Context, win: WindowCtx) -> Self {
        let mut glc = Rc::new(gg::GlContext::new(gl));
        unsafe {
            let (w, h): (u32, u32) = win.window().inner_size().into();
            let pass = gg::RenderPass::new_screen(
                Rc::get_mut(&mut glc).expect("Can't happen"),
                w as usize,
                h as usize,
                Some((0.1, 0.2, 0.3, 1.0)),
            );
            let screen_pass = RenderPass {
                inner: pass,
                gl: glc.clone(),
            };

            Self {
                glc: glc,
                win,
                screen_pass,
                passes: vec![],
            }
        }
    }

    /// Sets window mode from a WindowMode object.
    pub(crate) fn set_window_mode(&mut self, mode: WindowMode) -> GameResult {
        //use crate::conf::FullscreenType;
        // use glutin::dpi;
        let window = self.win.window();

        window.set_maximized(mode.maximized);

        // TODO: Min and max dimension constraints have gone away,
        // remove them from WindowMode

        //let monitors = window.available_monitors();
        // TODO: Okay, how we set fullscreen stuff has changed
        // and this needs to be totally revamped.
        /*
        match (mode.fullscreen_type, monitors.last()) {
            (FullscreenType::Windowed, _) => {
                window.set_fullscreen(None);
                window.set_decorations(!mode.borderless);
                window.set_inner_size(dpi::LogicalSize {
                    width: f64::from(mode.width),
                    height: f64::from(mode.height),
                });
                window.set_resizable(mode.resizable);
            }
            (FullscreenType::True, Some(monitor)) => {
                window.set_fullscreen(Some(monitor));
                window.set_inner_size(dpi::LogicalSize {
                    width: f64::from(mode.width),
                    height: f64::from(mode.height),
                });
            }
            (FullscreenType::Desktop, Some(monitor)) => {
                let position = monitor.get_position();
                let dimensions = monitor.get_dimensions();
                let hidpi_factor = window.get_hidpi_factor();
                window.set_fullscreen(None);
                window.set_decorations(false);
                window.set_inner_size(dimensions.to_logical(hidpi_factor));
                window.set_position(position.to_logical(hidpi_factor));
            }
            _ => panic!("Unable to detect monitor; if you are on Linux Wayland it may be this bug: https://github.com/rust-windowing/winit/issues/793"),
        }
        */
        Ok(())
    }

    // TODO
    pub(crate) fn resize_viewport(&mut self) {
        let physical_size = self.win.window().inner_size();
        assert!(physical_size.width <= std::i32::MAX as u32);
        assert!(physical_size.height <= std::i32::MAX as u32);
        self.screen_pass.inner.set_viewport(
            0,
            0,
            physical_size.width as i32,
            physical_size.height as i32,
        );
    }
}

pub trait WindowTrait {
    fn request_redraw(&self);
    fn swap_buffers(&self);
}

/// Used for desktop
///
/// TODO: Wait... why is this different for wasm and not-wasm again?
/// Oh, I think it's 'cause glutin doesn't quite handle wasm right yet.
#[cfg(not(target_arch = "wasm32"))]
impl WindowTrait for glutin::WindowedContext<glutin::PossiblyCurrent> {
    fn request_redraw(&self) {
        self.window().request_redraw();
    }
    fn swap_buffers(&self) {
        self.swap_buffers().unwrap();
    }
}

/// Used for wasm
#[cfg(target_arch = "wasm32")]
impl WindowTrait for winit::window::Window {
    fn request_redraw(&self) {
        self.request_redraw();
    }
    fn swap_buffers(&self) {
        /*
        let msg = format!("swapped buffers");
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&msg));
        */
    }
}

// **********************************************************************
// DRAWING
// **********************************************************************

/// Basically a render pipeline with some useful fluff around it.
/// Handles auto-batching of DrawCall's so you don't have to.
#[derive(Debug)]
pub struct QuadBatch {
    current_image: Option<Image>,
    current_sampler: Option<SamplerSpec>,
    pipe: gg::QuadPipeline,
}

impl QuadBatch {
    /// Make a new QuadBatch with the default shader.
    pub fn new(ctx: &mut Context, projection: Matrix4) -> Self {
        let shader = {
            let gl = gl_context(ctx);
            gl.default_shader()
        };
        Self::new_with_shader(ctx, projection, shader)
    }

    /// Make a new QuadBatch with the given shader.
    pub fn new_with_shader(ctx: &mut Context, projection: Matrix4, shader: gg::Shader) -> Self {
        let gl = gl_context(ctx);
        let pipe = unsafe { gg::QuadPipeline::new(gl.clone(), shader, projection) };
        Self {
            current_image: None,
            current_sampler: None,
            pipe,
        }
    }

    pub fn set_projection(&mut self, projection: Matrix4) {
        self.pipe.projection = projection
    }

    /// Add the given quad to the draw batch, with the given image and sampler.
    pub fn add(&mut self, image: &Image, sampler: SamplerSpec, quad: gg::QuadData) {
        use gg::Pipeline;
        // TODO: Can we clean this up some?  Just comparing the raw image and sampler
        // is weird.
        let dc = if self.current_image.as_ref() == Some(image)
            && self.current_sampler == Some(sampler)
        {
            // We add to the current draw call
            self.pipe.drawcalls.last_mut().expect("can't happen")
        } else {
            // We create a new drawcall.
            self.current_image = Some(image.clone());
            self.current_sampler = Some(sampler);
            self.pipe.new_drawcall(image.texture.clone(), sampler)
        };
        dc.add(quad);
    }

    /// Add with default sampler
    pub fn add_quad(&mut self, image: &Image, quad: gg::QuadData) {
        self.add(image, SamplerSpec::default(), quad);
    }
}

#[derive(Debug)]
pub struct RenderPass {
    inner: gg::RenderPass,
    gl: Rc<gg::GlContext>,
}

impl RenderPass {
    /// Draw the given `QuadBatch`es to the render pass's draw target.
    /// Does not clear the target first.
    pub fn draw(&mut self, batches: &mut [QuadBatch]) {
        // TODO: Can we do this in terms of clear_draw()?  Not without making it
        // take an Option<Color> I suppose, which is a little redundant.  Alas.
        self.inner.set_clear_color(None);
        self.inner
            .draw(&*self.gl, batches.iter_mut().map(|b| &mut b.pipe));
    }

    /// Clears the render target to the given color, then draws the given
    /// draw batches on it.
    /// (The clear and draw kinda can't be separate methods for Reasons, they both
    /// ideally have to be done at once.)
    ///
    /// Set to `None` to not clear it at all.
    pub fn clear_draw(&mut self, color: Color, batches: &mut [QuadBatch]) {
        let color = Some((color.r, color.g, color.b, color.a));
        self.inner.set_clear_color(color);
        self.inner
            .draw(&*self.gl, batches.iter_mut().map(|b| &mut b.pipe));
    }
}

/// TODO: Clean up
pub fn default_shader(ctx: &Context) -> gg::Shader {
    let gl = gl_context(ctx);
    gl.default_shader()
}

/// TODO: Figure out and clean up.
/// This returns an ortho projection where 1 pixel == 1 unit,
/// with the origin at top-left and Y increasing downwards.
/// It always returns the correct projection for the current size
/// of the window.
pub fn default_projection(ctx: &Context) -> Matrix4 {
    let (w, h) = drawable_size(ctx);

    // TODO: Why do we need to transpose this?
    // It's a BUGGO either in our shader or in the glam function (which I wrote)
    Matrix4::orthographic_rh_gl(0.0, w, h, 0.0, -1.0, 1.0).transpose()
}

/// Returns the render pass that draws to the screen.
pub fn screen_pass(ctx: &mut Context) -> &mut RenderPass {
    &mut ctx.gfx_context.screen_pass
}

/// Tells the graphics system to actually put everything on the screen.
/// Call this at the end of your [`EventHandler`](../event/trait.EventHandler.html)'s
/// [`draw()`](../event/trait.EventHandler.html#tymethod.draw) method.
///
/// Unsets any active canvas.
pub fn present(ctx: &mut Context) -> GameResult<()> {
    ctx.gfx_context.win.swap_buffers()?;
    Ok(())
}

// **********************************************************************
// GRAPHICS STATE
// **********************************************************************

/// Returns a string that tells a little about the obtained rendering mode.
/// It is supposed to be human-readable and will change; do not try to parse
/// information out of it!
pub fn renderer_info(ctx: &Context) -> GameResult<String> {
    let (vend, rend, vers, shader_vers) = ctx.gfx_context.glc.get_info();
    Ok(format!(
        "GL context created info:
  Vendor: {}
  Renderer: {}
  Version: {}
  Shader version: {}",
        vend, rend, vers, shader_vers
    ))
}

/// Sets the window mode, such as the size and other properties.
///
/// Setting the window mode may have side effects, such as clearing
/// the screen or setting the screen coordinates viewport to some
/// undefined value (for example, the window was resized).  It is
/// recommended to call
/// [`set_screen_coordinates()`](fn.set_screen_coordinates.html) after
/// changing the window size to make sure everything is what you want
/// it to be.
pub fn set_mode(context: &mut Context, mode: WindowMode) -> GameResult {
    let gfx = &mut context.gfx_context;
    gfx.set_window_mode(mode)?;
    // Save updated mode.
    context.conf.window_mode = mode;
    Ok(())
}

// TODO
// /// Sets the window icon.
// pub fn set_window_icon<P: AsRef<Path>>(context: &mut Context, path: Option<P>) -> GameResult<()> {
//     let icon = match path {
//         Some(p) => {
//             let p: &Path = p.as_ref();
//             Some(context::load_icon(p, &mut context.filesystem)?)
//         }
//         None => None,
//     };
//     context.gfx_context.window.set_window_icon(icon);
//     Ok(())
// }

/// Sets the window title.
pub fn set_window_title(context: &Context, title: &str) {
    context.gfx_context.win.window().set_title(title);
}

/// Returns the size of the window in pixels as (width, height),
/// including borders, titlebar, etc.
/// Returns zeros if the window doesn't exist.
pub fn size(context: &Context) -> (f32, f32) {
    let gfx = &context.gfx_context;
    let size = gfx.win.window().outer_size();
    (size.width as f32, size.height as f32)
}

/// Returns the size of the window's underlying drawable in pixels as (width, height).
/// Returns zeros if window doesn't exist.
pub fn drawable_size(context: &Context) -> (f32, f32) {
    let gfx = &context.gfx_context;
    let size = gfx.win.window().inner_size();
    (size.width as f32, size.height as f32)
}

/// Sets the window to fullscreen or back.
pub fn set_fullscreen(context: &mut Context, fullscreen: conf::FullscreenType) -> GameResult {
    let mut window_mode = context.conf.window_mode;
    window_mode.fullscreen_type = fullscreen;
    set_mode(context, window_mode)
}

/// Sets the window size/resolution to the specified width and height.
pub fn set_drawable_size(context: &mut Context, width: f32, height: f32) -> GameResult {
    let mut window_mode = context.conf.window_mode;
    window_mode.width = width;
    window_mode.height = height;
    set_mode(context, window_mode)
}

/// Sets whether or not the window is resizable.
pub fn set_resizable(context: &mut Context, resizable: bool) -> GameResult {
    let mut window_mode = context.conf.window_mode;
    window_mode.resizable = resizable;
    set_mode(context, window_mode)
}

/// Returns a reference to the Glutin window.
/// Ideally you should not need to use this because ggez
/// would provide all the functions you need without having
/// to dip into Glutin itself.  But life isn't always ideal.
pub fn window(context: &Context) -> &glutin::window::Window {
    let gfx = &context.gfx_context;
    &gfx.win.window()
}

/// TODO: This is roughb ut works
pub fn gl_context(context: &Context) -> Rc<gg::GlContext> {
    context.gfx_context.glc.clone()
}

/*


/// Take a screenshot by outputting the current render surface
/// (screen or selected canvas) to an `Image`.
pub fn screenshot(ctx: &mut Context) -> GameResult<Image> {
    // TODO LATER: This makes the screenshot upside-down form some reason...
    // Probably because all our images are upside down, for coordinate reasons!
    // How can we fix it?
    use gfx::memory::Bind;
    let debug_id = DebugId::get(ctx);

    let gfx = &mut ctx.gfx_context;
    let (w, h, _depth, aa) = gfx.data.out.get_dimensions();
    let surface_format = gfx.color_format();
    let gfx::format::Format(surface_type, channel_type) = surface_format;

    let texture_kind = gfx::texture::Kind::D2(w, h, aa);
    let texture_info = gfx::texture::Info {
        kind: texture_kind,
        levels: 1,
        format: surface_type,
        bind: Bind::TRANSFER_SRC | Bind::TRANSFER_DST | Bind::SHADER_RESOURCE,
        usage: gfx::memory::Usage::Data,
    };
    let target_texture = gfx
        .factory
        .create_texture_raw(texture_info, Some(channel_type), None)?;
    let image_info = gfx::texture::ImageInfoCommon {
        xoffset: 0,
        yoffset: 0,
        zoffset: 0,
        width: w,
        height: h,
        depth: 0,
        format: surface_format,
        mipmap: 0,
    };

    let mut local_encoder: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer> =
        gfx.factory.create_command_buffer().into();

    local_encoder.copy_texture_to_texture_raw(
        gfx.data.out.get_texture(),
        None,
        image_info,
        &target_texture,
        None,
        image_info,
    )?;

    local_encoder.flush(&mut *gfx.device);

    let resource_desc = gfx::texture::ResourceDesc {
        channel: channel_type,
        layer: None,
        min: 0,
        max: 0,
        swizzle: gfx::format::Swizzle::new(),
    };
    let shader_resource = gfx
        .factory
        .view_texture_as_shader_resource_raw(&target_texture, resource_desc)?;
    let image = Image {
        texture: shader_resource,
        texture_handle: target_texture,
        sampler_info: gfx.default_sampler_info,
        blend_mode: None,
        width: w,
        height: h,
        debug_id,
    };

    Ok(image)
}

// **********************************************************************
// GRAPHICS STATE
// **********************************************************************

/// Returns a rectangle defining the coordinate system of the screen.
/// It will be `Rect { x: left, y: top, w: width, h: height }`
///
/// If the Y axis increases downwards, the `height` of the `Rect`
/// will be negative.
pub fn screen_coordinates(ctx: &Context) -> Rect {
    ctx.gfx_context.screen_rect
}

/// Sets the bounds of the screen viewport.
///
/// The default coordinate system has (0,0) at the top-left corner
/// with X increasing to the right and Y increasing down, with the
/// viewport scaled such that one coordinate unit is one pixel on the
/// screen.  This function lets you change this coordinate system to
/// be whatever you prefer.
///
/// The `Rect`'s x and y will define the top-left corner of the screen,
/// and that plus its w and h will define the bottom-right corner.
pub fn set_screen_coordinates(context: &mut Context, rect: Rect) -> GameResult {
    let gfx = &mut context.gfx_context;
    gfx.set_projection_rect(rect);
    gfx.calculate_transform_matrix();
    gfx.update_globals()
}



/// Applies `DrawParam` to `Rect`.
pub fn transform_rect(rect: Rect, param: DrawParam) -> Rect {
    let w = param.src.w * param.scale.x * rect.w;
    let h = param.src.h * param.scale.y * rect.h;
    let offset_x = w * param.offset.x;
    let offset_y = h * param.offset.y;
    let dest_x = param.dest.x - offset_x;
    let dest_y = param.dest.y - offset_y;
    let mut r = Rect {
        w,
        h,
        x: dest_x + rect.x * param.scale.x,
        y: dest_y + rect.y * param.scale.y,
    };
    r.rotate(param.rotation);
    r
}

#[cfg(test)]
mod tests {
    use crate::graphics::{transform_rect, DrawParam, Rect};
    use approx::assert_relative_eq;
    use std::f32::consts::PI;

    #[test]
    fn headless_test_transform_rect() {
        {
            let r = Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            };
            let param = DrawParam::new();
            let real = transform_rect(r, param);
            let expected = r;
            assert_relative_eq!(real, expected);
        }
        {
            let r = Rect {
                x: -1.0,
                y: -1.0,
                w: 2.0,
                h: 1.0,
            };
            let param = DrawParam::new().scale([0.5, 0.5]);
            let real = transform_rect(r, param);
            let expected = Rect {
                x: -0.5,
                y: -0.5,
                w: 1.0,
                h: 0.5,
            };
            assert_relative_eq!(real, expected);
        }
        {
            let r = Rect {
                x: -1.0,
                y: -1.0,
                w: 1.0,
                h: 1.0,
            };
            let param = DrawParam::new().offset([0.5, 0.5]);
            let real = transform_rect(r, param);
            let expected = Rect {
                x: -1.5,
                y: -1.5,
                w: 1.0,
                h: 1.0,
            };
            assert_relative_eq!(real, expected);
        }
        {
            let r = Rect {
                x: 1.0,
                y: 0.0,
                w: 2.0,
                h: 1.0,
            };
            let param = DrawParam::new().rotation(PI * 0.5);
            let real = transform_rect(r, param);
            let expected = Rect {
                x: -1.0,
                y: 1.0,
                w: 1.0,
                h: 2.0,
            };
            assert_relative_eq!(real, expected);
        }
        {
            let r = Rect {
                x: -1.0,
                y: -1.0,
                w: 2.0,
                h: 1.0,
            };
            let param = DrawParam::new()
                .scale([0.5, 0.5])
                .offset([0.0, 1.0])
                .rotation(PI * 0.5);
            let real = transform_rect(r, param);
            let expected = Rect {
                x: 0.5,
                y: -0.5,
                w: 0.5,
                h: 1.0,
            };
            assert_relative_eq!(real, expected);
        }
    }
}
*/
