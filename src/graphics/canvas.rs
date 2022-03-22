use crevice::std140::AsStd140;

use crate::GameResult;

use super::{
    gpu::arc::{ArcBindGroup, ArcBindGroupLayout},
    internal_canvas::InternalCanvas,
    BlendMode, Color, DrawParam, GraphicsContext, Image, InstanceArray, Mesh, Rect, Sampler,
    ScreenImage, Shader, ShaderParams, Text, TextLayout, ZIndex,
};
use std::collections::BTreeMap;

/// Canvases are the main method of drawing meshes and text to images in ggez.
///
/// They can draw to any image that is capable of being drawn to (i.e. has been created with [`Image::new_canvas_image()`] or [`ScreenImage`]),
/// or they can draw directly to the screen.
///
/// Canvases are also where you can bind your own custom shaders and samplers to use while drawing.
/// Canvases *do not* automatically batch draws. To used batched (instanced) drawing, refer to [`InstanceArray`].
#[derive(Debug)]
pub struct Canvas {
    draws: BTreeMap<ZIndex, Vec<DrawCommand>>,
    state: DrawState,
    defaults: DefaultResources,

    target: Image,
    resolve: Option<Image>,
    load_op: CanvasLoadOp,
}

impl Canvas {
    /// Create a new [Canvas] from an image. This will allow for drawing to a single color image.
    ///
    /// The image must be created for Canvas usage, i.e. [Image::new_canvas_image()], or [ScreenImage], and must only have a sample count of 1.
    pub fn from_image(
        gfx: &GraphicsContext,
        image: Image,
        load_op: impl Into<CanvasLoadOp>,
    ) -> Self {
        Canvas::new(gfx, image, None, load_op.into())
    }

    /// Helper for [`Canvas::from_image`] for construction of a [`Canvas`] from a [`ScreenImage`].
    pub fn from_screen_image(
        gfx: &GraphicsContext,
        image: &mut ScreenImage,
        load_op: impl Into<CanvasLoadOp>,
    ) -> Self {
        let image = image.image(gfx);
        Canvas::from_image(gfx, image, load_op)
    }

    /// Create a new [Canvas] from an MSAA image and a resolve target. This will allow for drawing with MSAA to a color image, then resolving the samples into a secondary target.
    ///
    /// Both images must be created for Canvas usage (see [Canvas::from_image]). `msaa_image` must have a sample count > 1 and `resolve_image` must strictly have a sample count of 1.
    pub fn from_msaa(
        gfx: &GraphicsContext,
        msaa_image: Image,
        resolve: Image,
        load_op: impl Into<CanvasLoadOp>,
    ) -> Self {
        Canvas::new(gfx, msaa_image, Some(resolve), load_op.into())
    }

    /// Helper for [`Canvas::from_msaa`] for construction of an MSAA [`Canvas`] from a [`ScreenImage`].
    pub fn from_screen_msaa(
        gfx: &GraphicsContext,
        msaa_image: &mut ScreenImage,
        resolve: &mut ScreenImage,
        load_op: impl Into<CanvasLoadOp>,
    ) -> Self {
        let msaa = msaa_image.image(gfx);
        let resolve = resolve.image(gfx);
        Canvas::from_msaa(gfx, msaa, resolve, load_op)
    }

    /// Create a new [Canvas] that renders directly to the window surface.
    pub fn from_frame(gfx: &GraphicsContext, load_op: impl Into<CanvasLoadOp>) -> Self {
        // these unwraps will never fail
        let samples = gfx.frame_msaa_image.as_ref().unwrap().samples();
        let (target, resolve) = if samples > 1 {
            (
                gfx.frame_msaa_image.clone().unwrap(),
                Some(gfx.frame_image.clone().unwrap()),
            )
        } else {
            (gfx.frame_image.clone().unwrap(), None)
        };
        Canvas::new(gfx, target, resolve, load_op.into())
    }

    fn new(
        gfx: &GraphicsContext,
        target: Image,
        resolve: Option<Image>,
        load_op: CanvasLoadOp,
    ) -> Self {
        let defaults = DefaultResources::new(gfx);

        let state = DrawState {
            shader: defaults.shader.clone(),
            params: None,
            text_shader: defaults.text_shader.clone(),
            text_params: None,
            sampler: Sampler::linear_clamp(),
            blend_mode: BlendMode::ALPHA,
            projection: glam::Mat4::IDENTITY.into(),
        };

        let mut this = Canvas {
            draws: BTreeMap::new(),
            state,
            defaults,

            target,
            resolve,
            load_op,
        };

        let size = gfx.drawable_size();
        this.set_screen_coordinates(Rect {
            x: 0.,
            y: 0.,
            w: size.0,
            h: size.1,
        });

        this
    }

    /// Sets the shader to use when drawing meshes.
    pub fn set_shader(&mut self, shader: Shader) {
        self.state.shader = shader;
    }

    /// Sets the shader parameters to use when drawing meshes.
    ///
    /// **Bound to bind group 3 for non-instanced draws, and 4 for instanced draws.**
    pub fn set_shader_params<Uniforms: AsStd140>(&mut self, params: ShaderParams<Uniforms>) {
        self.state.params = Some((params.bind_group.clone(), params.layout));
    }

    /// Sets the shader to use when drawing text.
    pub fn set_text_shader(&mut self, shader: Shader) {
        self.state.text_shader = shader;
    }

    /// Sets the shader parameters to use when drawing text.
    ///
    /// **Bound to bind group 3.**
    pub fn set_text_shader_params<Uniforms: AsStd140>(&mut self, params: ShaderParams<Uniforms>) {
        self.state.text_params = Some((params.bind_group.clone(), params.layout));
    }

    /// Resets the active mesh shader to the default.
    pub fn set_default_shader(&mut self) {
        self.state.shader = self.defaults.shader.clone();
    }

    /// Resets the active text shader to the default.
    pub fn set_default_text_shader(&mut self) {
        self.state.text_shader = self.defaults.text_shader.clone();
    }

    /// Sets the active sampler used to sample images.
    pub fn set_sampler(&mut self, sampler: Sampler) {
        self.state.sampler = sampler;
    }

    /// Resets the active sampler to the default.
    ///
    /// This is equivalent to `set_sampler(Sampler::linear_clamp())`.
    pub fn set_default_sampler(&mut self) {
        self.set_sampler(Sampler::linear_clamp());
    }

    /// Sets the active blend mode used when drawing images.
    pub fn set_blend_mode(&mut self, blend_mode: BlendMode) {
        self.state.blend_mode = blend_mode;
    }

    /// Gets a copy of the canvas's raw projection matrix.
    #[inline]
    pub fn projection(&self) -> mint::ColumnMatrix4<f32> {
        self.state.projection
    }

    /// Sets the bounds of the screen viewport. This is a shortcut for `set_projection`
    /// and thus will override any previous projection matrix set.
    ///
    /// The default coordinate system has (0,0) at the top-left corner
    /// with X increasing to the right and Y increasing down, with the
    /// viewport scaled such that one coordinate unit is one pixel on the
    /// screen.  This function lets you change this coordinate system to
    /// be whatever you prefer.
    ///
    /// The `Rect`'s x and y will define the top-left corner of the screen,
    /// and that plus its w and h will define the bottom-right corner.
    pub fn set_screen_coordinates(&mut self, rect: Rect) {
        self.set_projection(screen_to_mat(rect));
    }

    /// Sets the raw projection matrix to the given homogeneous
    /// transformation matrix.  For an introduction to graphics matrices,
    /// a good source is this: <http://ncase.me/matrix/>
    pub fn set_projection(&mut self, proj: impl Into<mint::ColumnMatrix4<f32>>) {
        self.state.projection = proj.into();
    }

    /// Premultiplies the given transformation matrix with the current projection matrix.
    pub fn mul_projection(&mut self, transform: impl Into<mint::ColumnMatrix4<f32>>) {
        self.set_projection(
            glam::Mat4::from(transform.into()) * glam::Mat4::from(self.state.projection),
        );
    }

    /// Draws a mesh.
    ///
    /// If no [`Image`] is given then the image color will be white.
    pub fn draw_mesh(
        &mut self,
        mesh: Mesh,
        image: impl Into<Option<Image>>,
        param: impl Into<DrawParam>,
    ) {
        let param = param.into();
        self.draws.entry(param.z).or_default().push(DrawCommand {
            state: self.state.clone(),
            draw: Draw::Mesh {
                mesh,
                image: image.into().unwrap_or_else(|| self.defaults.image.clone()),
                param,
            },
        });
    }

    /// Draws a rectangle with a given [`Image`] and [`DrawParam`].
    ///
    /// Also see [`Canvas::draw_mesh()`].
    pub fn draw(&mut self, image: impl Into<Option<Image>>, param: impl Into<DrawParam>) {
        let param = param.into();
        self.draws.entry(param.z).or_default().push(DrawCommand {
            state: self.state.clone(),
            draw: Draw::Mesh {
                mesh: self.defaults.mesh.clone(),
                image: image.into().unwrap_or_else(|| self.defaults.image.clone()),
                param,
            },
        });
    }

    /// Draws a mesh instanced many times, using the [DrawParam]s found in `instances`.
    ///
    /// If no [`Image`] is given then the image color will be white.
    pub fn draw_mesh_instances(
        &mut self,
        mesh: Mesh,
        instances: InstanceArray,
        param: impl Into<DrawParam>,
    ) {
        let param = param.into();
        self.draws.entry(param.z).or_default().push(DrawCommand {
            state: self.state.clone(),
            draw: Draw::MeshInstances {
                mesh,
                instances,
                param,
            },
        });
    }

    /// Draws a rectangle instanced multiple times, as defined by the given [`InstanceArray`].
    ///
    /// Also see [`Canvas::draw_mesh_instances()`].
    pub fn draw_instances(&mut self, instances: InstanceArray, param: impl Into<DrawParam>) {
        let param = param.into();
        self.draws.entry(param.z).or_default().push(DrawCommand {
            state: self.state.clone(),
            draw: Draw::MeshInstances {
                mesh: self.defaults.mesh.clone(),
                instances,
                param,
            },
        });
    }

    /// Draws a section text that is fit and aligned into a given `rect` bounds.
    ///
    /// The section can be made up of multiple [Text], letting the user have complex formatting
    /// in the same section of text (e.g. bolding, highlighting, headers, etc).
    ///
    /// [TextLayout] determines how the text is aligned in `rect` and whether the text wraps or not.
    ///
    /// ## A tip for performance
    /// Text rendering will automatically batch *as long as the text draws are consecutive*.
    /// As such, to achieve the best performance, do all your text rendering in a single burst.
    pub fn draw_bounded_text(
        &mut self,
        text: &[Text],
        rect: Rect,
        rotation: f32,
        layout: TextLayout,
        z: ZIndex,
    ) {
        self.draws.entry(z).or_default().push(DrawCommand {
            state: self.state.clone(),
            draw: Draw::BoundedText {
                text: text.to_vec(),
                rect,
                rotation,
                layout,
            },
        });
    }

    /// Unbounded version of [`Canvas::draw_bounded_text()`].
    pub fn draw_text(
        &mut self,
        text: &[Text],
        pos: impl Into<mint::Vector2<f32>>,
        rotation: f32,
        layout: TextLayout,
        z: ZIndex,
    ) {
        let pos = pos.into();
        self.draw_bounded_text(
            text,
            Rect::new(pos.x, pos.y, f32::INFINITY, f32::INFINITY),
            rotation,
            layout,
            z,
        )
    }

    /// Finish drawing with this canvas and submit all the draw calls.
    pub fn finish(mut self, gfx: &mut GraphicsContext) -> GameResult {
        self.finalize(gfx)
    }

    fn finalize(&mut self, gfx: &mut GraphicsContext) -> GameResult {
        let mut canvas = if let Some(resolve) = &self.resolve {
            InternalCanvas::from_msaa(gfx, self.load_op, &self.target, resolve)?
        } else {
            InternalCanvas::from_image(gfx, self.load_op, &self.target)?
        };

        let mut state = self.state.clone();

        // apply initial state
        canvas.set_shader(state.shader.clone());
        if let Some((bind_group, layout)) = &state.params {
            canvas.set_shader_params(bind_group.clone(), layout.clone());
        }

        canvas.set_text_shader(state.text_shader.clone());
        if let Some((bind_group, layout)) = &state.text_params {
            canvas.set_text_shader_params(bind_group.clone(), layout.clone());
        }

        canvas.set_sampler(state.sampler);
        canvas.set_blend_mode(state.blend_mode);
        canvas.set_projection(state.projection);

        for draws in self.draws.values() {
            for draw in draws {
                if draw.state.shader != state.shader {
                    canvas.set_shader(state.shader.clone());
                }

                if draw.state.params != state.params {
                    if let Some((bind_group, layout)) = &draw.state.params {
                        canvas.set_shader_params(bind_group.clone(), layout.clone());
                    }
                }

                if draw.state.text_shader != state.text_shader {
                    canvas.set_text_shader(state.text_shader.clone());
                }

                if draw.state.text_params != state.text_params {
                    if let Some((bind_group, layout)) = &draw.state.text_params {
                        canvas.set_text_shader_params(bind_group.clone(), layout.clone());
                    }
                }

                if draw.state.sampler != state.sampler {
                    canvas.set_sampler(draw.state.sampler);
                }

                if draw.state.blend_mode != state.blend_mode {
                    canvas.set_blend_mode(draw.state.blend_mode);
                }

                if draw.state.projection != state.projection {
                    canvas.set_projection(draw.state.projection);
                }

                state = draw.state.clone();

                match &draw.draw {
                    Draw::Mesh { mesh, image, param } => canvas.draw_mesh(mesh, image, *param),
                    Draw::MeshInstances {
                        mesh,
                        instances,
                        param,
                    } => canvas.draw_mesh_instances(mesh, instances, *param),
                    Draw::BoundedText {
                        text,
                        rect,
                        rotation,
                        layout,
                    } => canvas.draw_bounded_text(text, *rect, *rotation, *layout)?,
                }
            }
        }

        canvas.finish();

        Ok(())
    }
}

/// Describes the image load operation when starting a new canvas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CanvasLoadOp {
    /// Keep the existing contents of the image.
    DontClear,
    /// Clear the image contents to a solid color.
    Clear(Color),
}

impl From<Option<Color>> for CanvasLoadOp {
    fn from(color: Option<Color>) -> Self {
        match color {
            Some(color) => CanvasLoadOp::Clear(color),
            None => CanvasLoadOp::DontClear,
        }
    }
}

impl From<Color> for CanvasLoadOp {
    #[inline]
    fn from(color: Color) -> Self {
        CanvasLoadOp::Clear(color)
    }
}

#[derive(Debug, Clone)]
struct DrawState {
    shader: Shader,
    params: Option<(ArcBindGroup, ArcBindGroupLayout)>,
    text_shader: Shader,
    text_params: Option<(ArcBindGroup, ArcBindGroupLayout)>,
    sampler: Sampler,
    blend_mode: BlendMode,
    projection: mint::ColumnMatrix4<f32>,
}

#[derive(Debug)]
enum Draw {
    Mesh {
        mesh: Mesh,
        image: Image,
        param: DrawParam,
    },
    MeshInstances {
        mesh: Mesh,
        instances: InstanceArray,
        param: DrawParam,
    },
    BoundedText {
        text: Vec<Text>,
        rect: Rect,
        rotation: f32,
        layout: TextLayout,
    },
}

#[derive(Debug)]
struct DrawCommand {
    state: DrawState,
    draw: Draw,
}

#[derive(Debug)]
struct DefaultResources {
    shader: Shader,
    text_shader: Shader,
    mesh: Mesh,
    image: Image,
}

impl DefaultResources {
    fn new(gfx: &GraphicsContext) -> Self {
        let shader = Shader {
            fragment: gfx.draw_shader.clone(),
            fs_entry: "fs_main".into(),
        };

        let text_shader = Shader {
            fragment: gfx.text_shader.clone(),
            fs_entry: "fs_main".into(),
        };

        let mesh = gfx.rect_mesh.clone();
        let image = gfx.white_image.clone();

        DefaultResources {
            shader,
            text_shader,
            mesh,
            image,
        }
    }
}

fn screen_to_mat(screen: Rect) -> glam::Mat4 {
    glam::Mat4::orthographic_rh(
        screen.left(),
        screen.right(),
        screen.bottom(),
        screen.top(),
        0.,
        1.,
    )
}
