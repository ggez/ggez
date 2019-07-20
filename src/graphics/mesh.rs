use crate::context::DebugId;
use crate::error::GameError;
use crate::graphics::*;
use gfx::traits::FactoryExt;
use lyon;
use lyon::tessellation as t;

pub use self::t::{FillOptions, FillRule, LineCap, LineJoin, StrokeOptions};

/// A builder for creating [`Mesh`](struct.Mesh.html)es.
///
/// This allows you to easily make one `Mesh` containing
/// many different complex pieces of geometry.  They don't
/// have to be connected to each other, and will all be
/// drawn at once.
///
/// The following example shows how to build a mesh containing a line and a circle:
///
/// ```rust,no_run
/// # use ggez::*;
/// # use ggez::graphics::*;
/// # use ggez::nalgebra::Point2;
/// # fn main() -> GameResult {
/// # let ctx = &mut ContextBuilder::new("foo", "bar").build().unwrap().0;
/// let mesh: Mesh = MeshBuilder::new()
///     .line(&[Point2::new(20.0, 20.0), Point2::new(40.0, 20.0)], 4.0, (255, 0, 0).into())?
///     .circle(DrawMode::fill(), Point2::new(60.0, 38.0), 40.0, 1.0, (0, 255, 0).into())
///     .build(ctx)?;
/// # Ok(()) }
/// ```
/// A more sophisticated example:
///
/// ```rust,no_run
/// use ggez::{Context, GameResult, nalgebra as na};
/// use ggez::graphics::{self, DrawMode, MeshBuilder};
///
/// fn draw_danger_signs(ctx: &mut Context) -> GameResult {
///     // Initialize a builder instance.
///     let mesh = MeshBuilder::new()
///         // Add vertices for 3 lines (in an approximate equilateral triangle).
///         .line(
///             &[
///                 na::Point2::new(0.0, 0.0),
///                 na::Point2::new(-30.0, 52.0),
///                 na::Point2::new(30.0, 52.0),
///                 na::Point2::new(0.0, 0.0),
///             ],
///             1.0,
///             graphics::WHITE,
///         )?
///         // Add vertices for an exclamation mark!
///         .ellipse(DrawMode::fill(), na::Point2::new(0.0, 25.0), 2.0, 15.0, 2.0, graphics::WHITE,)
///         .circle(DrawMode::fill(), na::Point2::new(0.0, 45.0), 2.0, 2.0, graphics::WHITE,)
///         // Finalize then unwrap. Unwrapping via `?` operator either yields the final `Mesh`,
///         // or propagates the error (note return type).
///         .build(ctx)?;
///     // Draw 3 meshes in a line, 1st and 3rd tilted by 1 radian.
///     graphics::draw(ctx, &mesh, (na::Point2::new(50.0, 50.0), -1.0, graphics::WHITE))?;
///     graphics::draw(ctx, &mesh, (na::Point2::new(150.0, 50.0), 0.0, graphics::WHITE))?;
///     graphics::draw(ctx, &mesh, (na::Point2::new(250.0, 50.0), 1.0, graphics::WHITE))?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct MeshBuilder {
    buffer: t::geometry_builder::VertexBuffers<Vertex, u32>,
    image: Option<Image>,
}

impl Default for MeshBuilder {
    fn default() -> Self {
        Self {
            buffer: t::VertexBuffers::new(),
            image: None,
        }
    }
}

impl MeshBuilder {
    /// Create a new `MeshBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new mesh for a line of one or more connected segments.
    pub fn line<P>(&mut self, points: &[P], width: f32, color: Color) -> GameResult<&mut Self>
    where
        P: Into<mint::Point2<f32>> + Clone,
    {
        self.polyline(DrawMode::stroke(width), points, color)
    }

    /// Create a new mesh for a circle.
    ///
    /// For the meaning of the `tolerance` parameter, [see here](https://docs.rs/lyon_geom/0.11.0/lyon_geom/#flattening).
    pub fn circle<P>(
        &mut self,
        mode: DrawMode,
        point: P,
        radius: f32,
        tolerance: f32,
        color: Color,
    ) -> &mut Self
    where
        P: Into<mint::Point2<f32>>,
    {
        {
            let point = point.into();
            let buffers = &mut self.buffer;
            let vb = VertexBuilder {
                color: LinearColor::from(color),
            };
            match mode {
                DrawMode::Fill(fill_options) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let _ = t::basic_shapes::fill_circle(
                        t::math::point(point.x, point.y),
                        radius,
                        &fill_options.with_tolerance(tolerance),
                        builder,
                    );
                }
                DrawMode::Stroke(options) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let _ = t::basic_shapes::stroke_circle(
                        t::math::point(point.x, point.y),
                        radius,
                        &options.with_tolerance(tolerance),
                        builder,
                    );
                }
            };
        }
        self
    }

    /// Create a new mesh for an ellipse.
    ///
    /// For the meaning of the `tolerance` parameter, [see here](https://docs.rs/lyon_geom/0.11.0/lyon_geom/#flattening).
    pub fn ellipse<P>(
        &mut self,
        mode: DrawMode,
        point: P,
        radius1: f32,
        radius2: f32,
        tolerance: f32,
        color: Color,
    ) -> &mut Self
    where
        P: Into<mint::Point2<f32>>,
    {
        {
            let buffers = &mut self.buffer;
            let point = point.into();
            let vb = VertexBuilder {
                color: LinearColor::from(color),
            };
            match mode {
                DrawMode::Fill(fill_options) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let _ = t::basic_shapes::fill_ellipse(
                        t::math::point(point.x, point.y),
                        t::math::vector(radius1, radius2),
                        t::math::Angle { radians: 0.0 },
                        &fill_options.with_tolerance(tolerance),
                        builder,
                    );
                }
                DrawMode::Stroke(options) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let _ = t::basic_shapes::stroke_ellipse(
                        t::math::point(point.x, point.y),
                        t::math::vector(radius1, radius2),
                        t::math::Angle { radians: 0.0 },
                        &options.with_tolerance(tolerance),
                        builder,
                    );
                }
            };
        }
        self
    }

    /// Create a new mesh for a series of connected lines.
    pub fn polyline<P>(
        &mut self,
        mode: DrawMode,
        points: &[P],
        color: Color,
    ) -> GameResult<&mut Self>
    where
        P: Into<mint::Point2<f32>> + Clone,
    {
        if points.len() < 2 {
            return Err(GameError::LyonError(
                "MeshBuilder::polyline() got a list of < 2 points".to_string(),
            ));
        }

        self.polyline_inner(mode, points, false, color)
    }

    /// Create a new mesh for a closed polygon.
    /// The points given must be in clockwise order,
    /// otherwise at best the polygon will not draw.
    pub fn polygon<P>(
        &mut self,
        mode: DrawMode,
        points: &[P],
        color: Color,
    ) -> GameResult<&mut Self>
    where
        P: Into<mint::Point2<f32>> + Clone,
    {
        if points.len() < 3 {
            return Err(GameError::LyonError(
                "MeshBuilder::polygon() got a list of < 3 points".to_string(),
            ));
        }

        self.polyline_inner(mode, points, true, color)
    }

    fn polyline_inner<P>(
        &mut self,
        mode: DrawMode,
        points: &[P],
        is_closed: bool,
        color: Color,
    ) -> GameResult<&mut Self>
    where
        P: Into<mint::Point2<f32>> + Clone,
    {
        {
            assert!(points.len() > 1);
            let buffers = &mut self.buffer;
            let points = points.iter().cloned().map(|p| {
                let mint_point: mint::Point2<f32> = p.into();
                t::math::point(mint_point.x, mint_point.y)
            });
            let vb = VertexBuilder {
                color: LinearColor::from(color),
            };
            match mode {
                DrawMode::Fill(options) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let tessellator = &mut t::FillTessellator::new();
                    let _ = t::basic_shapes::fill_polyline(points, tessellator, &options, builder)?;
                }
                DrawMode::Stroke(options) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let _ = t::basic_shapes::stroke_polyline(points, is_closed, &options, builder);
                }
            };
        }
        Ok(self)
    }

    /// Create a new mesh for a rectangle.
    pub fn rectangle(&mut self, mode: DrawMode, bounds: Rect, color: Color) -> &mut Self {
        {
            let buffers = &mut self.buffer;
            let rect = t::math::rect(bounds.x, bounds.y, bounds.w, bounds.h);
            let vb = VertexBuilder {
                color: LinearColor::from(color),
            };
            match mode {
                DrawMode::Fill(fill_options) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let _ = t::basic_shapes::fill_rectangle(&rect, &fill_options, builder);
                }
                DrawMode::Stroke(options) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let _ = t::basic_shapes::stroke_rectangle(&rect, &options, builder);
                }
            };
        }
        self
    }

    /// Create a new [`Mesh`](struct.Mesh.html) from a raw list of triangles.
    /// The length of the list must be a multiple of 3.
    ///
    /// Currently does not support UV's or indices.
    pub fn triangles<P>(&mut self, triangles: &[P], color: Color) -> GameResult<&mut Self>
    where
        P: Into<mint::Point2<f32>> + Clone,
    {
        {
            if (triangles.len() % 3) != 0 {
                let msg = format!(
                    "Called Mesh::triangles() with points that have a length not a multiple of 3."
                );
                return Err(GameError::LyonError(msg));
            }
            let tris = triangles
                .iter()
                .cloned()
                .map(|p| {
                    // Gotta turn ggez Point2's into lyon FillVertex's
                    let mint_point = p.into();
                    let np = lyon::math::point(mint_point.x, mint_point.y);
                    let nv = lyon::math::vector(mint_point.x, mint_point.y);
                    t::FillVertex {
                        position: np,
                        normal: nv,
                    }
                })
                // Removing this collect might be nice, but is not easy.
                // We can chunk a slice, but can't chunk an arbitrary
                // iterator.
                // Using the itertools crate doesn't really make anything
                // nicer, so we'll just live with it.
                .collect::<Vec<_>>();
            let tris = tris.chunks(3);
            let vb = VertexBuilder {
                color: LinearColor::from(color),
            };
            let builder: &mut t::BuffersBuilder<_, _, _, _> =
                &mut t::BuffersBuilder::new(&mut self.buffer, vb);
            use lyon::tessellation::GeometryBuilder;
            builder.begin_geometry();
            for tri in tris {
                // Ideally this assert makes bounds-checks only happen once.
                assert!(tri.len() == 3);
                let fst = tri[0];
                let snd = tri[1];
                let thd = tri[2];
                let i1 = builder.add_vertex(fst)?;
                let i2 = builder.add_vertex(snd)?;
                let i3 = builder.add_vertex(thd)?;
                builder.add_triangle(i1, i2, i3);
            }
            let _ = builder.end_geometry();
        }
        Ok(self)
    }

    /// Takes an `Image` to apply to the mesh.
    pub fn texture(&mut self, texture: Image) -> &mut Self {
        self.image = Some(texture);
        self
    }

    /// Creates a `Mesh` from a raw list of triangles defined from vertices
    /// and indices.  You may also
    /// supply an `Image` to use as a texture, if you pass `None`, it will
    /// just use a pure white texture.
    ///
    /// This is the most primitive mesh-creation method, but allows you full
    /// control over the tesselation and texturing.  It has the same constraints
    /// as `Mesh::from_raw()`.
    pub fn raw<V>(&mut self, verts: &[V], indices: &[u32], texture: Option<Image>) -> &mut Self
    where
        V: Into<Vertex> + Clone,
    {
        assert!(self.buffer.vertices.len() + verts.len() < (std::u32::MAX as usize));
        assert!(self.buffer.indices.len() + indices.len() < (std::u32::MAX as usize));
        let next_idx = self.buffer.vertices.len() as u32;
        // Can we remove the clone here?
        // I can't find a way to, because `into()` consumes its source and
        // `Borrow` or `AsRef` aren't really right.
        let vertices = verts.iter().cloned().map(|v: V| -> Vertex { v.into() });
        let indices = indices.iter().map(|i| (*i) + next_idx);
        self.buffer.vertices.extend(vertices);
        self.buffer.indices.extend(indices);
        self.image = texture;
        self
    }

    /// Takes the accumulated geometry and load it into GPU memory,
    /// creating a single `Mesh`.
    pub fn build(&self, ctx: &mut Context) -> GameResult<Mesh> {
        Mesh::from_raw(
            ctx,
            &self.buffer.vertices,
            &self.buffer.indices,
            self.image.clone(),
        )
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct VertexBuilder {
    color: LinearColor,
}

impl t::VertexConstructor<t::FillVertex, Vertex> for VertexBuilder {
    fn new_vertex(&mut self, vertex: t::FillVertex) -> Vertex {
        Vertex {
            pos: [vertex.position.x, vertex.position.y],
            uv: [vertex.position.x, vertex.position.y],
            color: self.color.into(),
        }
    }
}

impl t::VertexConstructor<t::StrokeVertex, Vertex> for VertexBuilder {
    fn new_vertex(&mut self, vertex: t::StrokeVertex) -> Vertex {
        Vertex {
            pos: [vertex.position.x, vertex.position.y],
            uv: [0.0, 0.0],
            color: self.color.into(),
        }
    }
}

/// 2D polygon mesh.
///
/// All of its creation methods are just shortcuts for doing the same operation
/// via a [`MeshBuilder`](struct.MeshBuilder.html).
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    buffer: gfx::handle::Buffer<gfx_device_gl::Resources, Vertex>,
    slice: gfx::Slice<gfx_device_gl::Resources>,
    blend_mode: Option<BlendMode>,
    image: Image,
    debug_id: DebugId,
    rect: Rect,
}

impl Mesh {
    /// Create a new mesh for a line of one or more connected segments.
    pub fn new_line<P>(
        ctx: &mut Context,
        points: &[P],
        width: f32,
        color: Color,
    ) -> GameResult<Mesh>
    where
        P: Into<mint::Point2<f32>> + Clone,
    {
        let mut mb = MeshBuilder::new();
        let _ = mb.polyline(DrawMode::stroke(width), points, color);
        mb.build(ctx)
    }

    /// Create a new mesh for a circle.
    pub fn new_circle<P>(
        ctx: &mut Context,
        mode: DrawMode,
        point: P,
        radius: f32,
        tolerance: f32,
        color: Color,
    ) -> GameResult<Mesh>
    where
        P: Into<mint::Point2<f32>>,
    {
        let mut mb = MeshBuilder::new();
        let _ = mb.circle(mode, point, radius, tolerance, color);
        mb.build(ctx)
    }

    /// Create a new mesh for an ellipse.
    pub fn new_ellipse<P>(
        ctx: &mut Context,
        mode: DrawMode,
        point: P,
        radius1: f32,
        radius2: f32,
        tolerance: f32,
        color: Color,
    ) -> GameResult<Mesh>
    where
        P: Into<mint::Point2<f32>>,
    {
        let mut mb = MeshBuilder::new();
        let _ = mb.ellipse(mode, point, radius1, radius2, tolerance, color);
        mb.build(ctx)
    }

    /// Create a new mesh for series of connected lines.
    pub fn new_polyline<P>(
        ctx: &mut Context,
        mode: DrawMode,
        points: &[P],
        color: Color,
    ) -> GameResult<Mesh>
    where
        P: Into<mint::Point2<f32>> + Clone,
    {
        let mut mb = MeshBuilder::new();
        let _ = mb.polyline(mode, points, color);
        mb.build(ctx)
    }

    /// Create a new mesh for closed polygon.
    /// The points given must be in clockwise order,
    /// otherwise at best the polygon will not draw.
    pub fn new_polygon<P>(
        ctx: &mut Context,
        mode: DrawMode,
        points: &[P],
        color: Color,
    ) -> GameResult<Mesh>
    where
        P: Into<mint::Point2<f32>> + Clone,
    {
        if points.len() < 3 {
            return Err(GameError::LyonError(
                "Mesh::new_polygon() got a list of < 3 points".to_string(),
            ));
        }
        let mut mb = MeshBuilder::new();
        let _ = mb.polygon(mode, points, color);
        mb.build(ctx)
    }

    /// Create a new mesh for a rectangle
    pub fn new_rectangle(
        ctx: &mut Context,
        mode: DrawMode,
        bounds: Rect,
        color: Color,
    ) -> GameResult<Mesh> {
        let mut mb = MeshBuilder::new();
        let _ = mb.rectangle(mode, bounds, color);
        mb.build(ctx)
    }

    /// Create a new `Mesh` from a raw list of triangle points.
    pub fn from_triangles<P>(ctx: &mut Context, triangles: &[P], color: Color) -> GameResult<Mesh>
    where
        P: Into<mint::Point2<f32>> + Clone,
    {
        let mut mb = MeshBuilder::new();
        let _ = mb.triangles(triangles, color);
        mb.build(ctx)
    }

    /// Creates a `Mesh` from a raw list of triangles defined from points
    /// and indices, with the given UV texture coordinates.  You may also
    /// supply an `Image` to use as a texture, if you pass `None`, it will
    /// just use a pure white texture.  The indices should draw the points in
    /// clockwise order, otherwise at best the mesh will not draw.
    ///
    /// This is the most primitive mesh-creation method, but allows
    /// you full control over the tesselation and texturing.  As such
    /// it will return an error, panic, or produce incorrect/invalid
    /// output (that may later cause drawing to panic), if:
    ///
    ///  * `indices` contains a value out of bounds of `verts`
    ///  * `verts` is longer than `u32::MAX` elements.
    ///  * `indices` do not specify triangles in clockwise order.
    pub fn from_raw<V>(
        ctx: &mut Context,
        verts: &[V],
        indices: &[u32],
        texture: Option<Image>,
    ) -> GameResult<Mesh>
    where
        V: Into<Vertex> + Clone,
    {
        // Sanity checks to return early with helpful error messages.
        if verts.len() > (std::u32::MAX as usize) {
            let msg = format!(
                "Tried to build a mesh with {} vertices, max is u32::MAX",
                verts.len()
            );
            return Err(GameError::LyonError(msg));
        }
        if indices.len() > (std::u32::MAX as usize) {
            let msg = format!(
                "Tried to build a mesh with {} indices, max is u32::MAX",
                indices.len()
            );
            return Err(GameError::LyonError(msg));
        }
        if verts.len() < 3 {
            let msg = format!("Trying to build mesh with < 3 vertices, this is usually due to invalid input to a `Mesh` or MeshBuilder`.");
            return Err(GameError::LyonError(msg));
        }
        if indices.len() < 3 {
            let msg = format!("Trying to build mesh with < 3 indices, this is usually due to invalid input to a `Mesh` or MeshBuilder`.  Indices:\n {:#?}", indices);
            return Err(GameError::LyonError(msg));
        }

        if indices.len() % 3 != 0 {
            let msg = format!("Trying to build mesh with an array of indices that is not a multiple of 3, this is usually due to invalid input to a `Mesh` or MeshBuilder`.");
            return Err(GameError::LyonError(msg));
        }

        let verts: Vec<Vertex> = verts.iter().cloned().map(Into::into).collect();
        let rect = bbox_for_vertices(&verts).expect("No vertices in MeshBuilder");
        let (vbuf, slice) = ctx
            .gfx_context
            .factory
            .create_vertex_buffer_with_slice(&verts[..], indices);
        Ok(Mesh {
            buffer: vbuf,
            slice,
            blend_mode: None,
            image: texture.unwrap_or_else(|| ctx.gfx_context.white_image.clone()),
            debug_id: DebugId::get(ctx),
            rect,
        })
    }

    /// Replaces the vertices in the `Mesh` with the given ones.  This MAY be faster
    /// than re-creating a `Mesh` with [`Mesh::from_raw()`](#method.from_raw) due to
    /// reusing memory instead of allocating and deallocating it, both on the CPU and
    /// GPU side.  There's too much variation in implementations and drivers to promise
    /// it will actually be faster though.  At worst, it will be the same speed.
    pub fn set_vertices(&mut self, ctx: &mut Context, verts: &[Vertex], indices: &[u32]) {
        // This is in principle faster than throwing away an existing mesh and
        // creating a new one with `Mesh::from_raw()`, but really only because it
        // doesn't take `Into<Vertex>` and so doesn't need to create an intermediate
        // `Vec`.  It still creates a new GPU buffer and replaces the old one instead
        // of just copying into the old one.
        //
        // By default we create `Mesh` with a read-only GPU buffer, which I am
        // a little hesitant to change... partially because doing that with
        // `Image` has caused some subtle edge case bugs.
        //
        // It's not terribly hard to do in principle though, just tedious;
        // start at `Factory::create_vertex_buffer_with_slice()`, drill down to
        // <https://docs.rs/gfx/0.17.1/gfx/traits/trait.Factory.html#tymethod.create_buffer_raw>,
        // and fill in the bits between with the appropriate values.
        let (vbuf, slice) = ctx
            .gfx_context
            .factory
            .create_vertex_buffer_with_slice(verts, indices);
        self.buffer = vbuf;
        self.slice = slice;
    }
}

impl Drawable for Mesh {
    fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult {
        self.debug_id.assert(ctx);
        let gfx = &mut ctx.gfx_context;
        gfx.update_instance_properties(param.into())?;

        gfx.data.vbuf = self.buffer.clone();
        let texture = self.image.texture.clone();
        let sampler = gfx
            .samplers
            .get_or_insert(self.image.sampler_info, gfx.factory.as_mut());

        let typed_thingy = gfx.backend_spec.raw_to_typed_shader_resource(texture);
        gfx.data.tex = (typed_thingy, sampler);

        gfx.draw(Some(&self.slice))?;

        Ok(())
    }
    fn dimensions(&self, _ctx: &mut Context) -> Option<Rect> {
        Some(self.rect)
    }
    fn set_blend_mode(&mut self, mode: Option<BlendMode>) {
        self.blend_mode = mode;
    }
    fn blend_mode(&self) -> Option<BlendMode> {
        self.blend_mode
    }
}

fn bbox_for_vertices(verts: &[Vertex]) -> Option<Rect> {
    if verts.is_empty() {
        return None;
    }
    let [x0, y0] = verts[0].pos;
    let mut x_max = x0;
    let mut x_min = x0;
    let mut y_max = y0;
    let mut y_min = y0;
    for v in verts {
        let x = v.pos[0];
        let y = v.pos[1];
        x_max = f32::max(x_max, x);
        x_min = f32::min(x_min, x);
        y_max = f32::max(y_max, y);
        y_min = f32::min(y_min, y);
    }
    Some(Rect {
        w: x_max - x_min,
        h: y_max - y_min,
        x: x_min,
        y: y_min,
    })
}
