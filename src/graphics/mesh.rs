use std::convert::TryInto;

use crate::context::DebugId;
use crate::error::GameError;
use crate::graphics::*;
use gfx::traits::FactoryExt;
use lyon::path::builder::PathBuilder;
use lyon::path::Polygon;
use lyon::tessellation as t;
use lyon::{self, math::Point as LPoint};

pub use self::t::{FillOptions, FillRule, LineCap, LineJoin, StrokeOptions};

/// A builder for creating [`Mesh`](struct.Mesh.html)es.
///
/// This allows you to easily make one `Mesh` containing
/// many different complex pieces of geometry.  They don't
/// have to be connected to each other, and will all be
/// drawn at once.
///
/// Note that this doesn't try very hard to handle degenerate cases.  It can easily break if you
/// tell it to do things that result in a circle of radius 0, a line of width 0, an infintessimally
/// skinny triangle, or other mathematically inconvenient things like that.
///
/// The following example shows how to build a mesh containing a line and a circle:
///
/// ```rust,no_run
/// # use ggez::*;
/// # use ggez::graphics::*;
/// # fn main() -> GameResult {
/// # let ctx = &mut ContextBuilder::new("foo", "bar").build().unwrap().0;
/// let mesh: Mesh = MeshBuilder::new()
///     .line(&[glam::vec2(20.0, 20.0), glam::vec2(40.0, 20.0)], 4.0, (255, 0, 0).into())?
///     .circle(DrawMode::fill(), glam::vec2(60.0, 38.0), 40.0, 1.0, (0, 255, 0).into())?
///     .build(ctx)?;
/// # Ok(()) }
/// ```
/// A more sophisticated example:
///
/// ```rust,no_run
/// use ggez::{Context, GameResult};
/// use ggez::graphics::{self, DrawMode, MeshBuilder};
///
/// fn draw_danger_signs(ctx: &mut Context) -> GameResult {
///     // Initialize a builder instance.
///     let mesh = MeshBuilder::new()
///         // Add vertices for 3 lines (in an approximate equilateral triangle).
///         .line(
///             &[
///                 glam::vec2(0.0, 0.0),
///                 glam::vec2(-30.0, 52.0),
///                 glam::vec2(30.0, 52.0),
///                 glam::vec2(0.0, 0.0),
///             ],
///             1.0,
///             graphics::Color::WHITE,
///         )?
///         // Add vertices for an exclamation mark!
///         .ellipse(DrawMode::fill(), glam::vec2(0.0, 25.0), 2.0, 15.0, 2.0, graphics::Color::WHITE,)?
///         .circle(DrawMode::fill(), glam::vec2(0.0, 45.0), 2.0, 2.0, graphics::Color::WHITE,)?
///         // Finalize then unwrap. Unwrapping via `?` operator either yields the final `Mesh`,
///         // or propagates the error (note return type).
///         .build(ctx)?;
///     // Draw 3 meshes in a line, 1st and 3rd tilted by 1 radian.
///     graphics::draw(ctx, &mesh, (glam::vec2(50.0, 50.0), -1.0, graphics::Color::WHITE))?;
///     graphics::draw(ctx, &mesh, (glam::vec2(150.0, 50.0), 0.0, graphics::Color::WHITE))?;
///     graphics::draw(ctx, &mesh, (glam::vec2(250.0, 50.0), 1.0, graphics::Color::WHITE))?;
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
    ) -> GameResult<&mut Self>
    where
        P: Into<mint::Point2<f32>>,
    {
        assert!(
            tolerance > 0.0,
            "Tolerances <= 0 are invalid, see https://github.com/ggez/ggez/issues/892"
        );
        {
            let point = point.into();
            let buffers = &mut self.buffer;
            let vb = VertexBuilder {
                color: LinearColor::from(color),
            };
            match mode {
                DrawMode::Fill(fill_options) => {
                    let mut tessellator = t::FillTessellator::new();
                    let _ = tessellator.tessellate_circle(
                        t::math::point(point.x, point.y),
                        radius,
                        &fill_options.with_tolerance(tolerance),
                        &mut t::BuffersBuilder::new(buffers, vb),
                    );
                }
                DrawMode::Stroke(options) => {
                    let mut tessellator = t::StrokeTessellator::new();
                    let _ = tessellator.tessellate_circle(
                        t::math::point(point.x, point.y),
                        radius,
                        &options.with_tolerance(tolerance),
                        &mut t::BuffersBuilder::new(buffers, vb),
                    );
                }
            };
        }
        Ok(self)
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
    ) -> GameResult<&mut Self>
    where
        P: Into<mint::Point2<f32>>,
    {
        assert!(
            tolerance > 0.0,
            "Tolerances <= 0 are invalid, see https://github.com/ggez/ggez/issues/892"
        );
        {
            let buffers = &mut self.buffer;
            let point = point.into();
            let vb = VertexBuilder {
                color: LinearColor::from(color),
            };
            match mode {
                DrawMode::Fill(fill_options) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let mut tessellator = t::FillTessellator::new();
                    let _ = tessellator.tessellate_ellipse(
                        t::math::point(point.x, point.y),
                        t::math::vector(radius1, radius2),
                        t::math::Angle { radians: 0.0 },
                        t::path::Winding::Positive,
                        &fill_options.with_tolerance(tolerance),
                        builder,
                    );
                }
                DrawMode::Stroke(options) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let mut tessellator = t::StrokeTessellator::new();
                    let _ = tessellator.tessellate_ellipse(
                        t::math::point(point.x, point.y),
                        t::math::vector(radius1, radius2),
                        t::math::Angle { radians: 0.0 },
                        t::path::Winding::Positive,
                        &options.with_tolerance(tolerance),
                        builder,
                    );
                }
            };
        }
        Ok(self)
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
        let vb = VertexBuilder {
            color: LinearColor::from(color),
        };
        self.polyline_with_vertex_builder(mode, points, is_closed, vb)
    }

    /// Create a new mesh for a given polyline using a custom vertex builder.
    /// The points given must be in clockwise order.
    pub fn polyline_with_vertex_builder<P, V>(
        &mut self,
        mode: DrawMode,
        points: &[P],
        is_closed: bool,
        vb: V,
    ) -> GameResult<&mut Self>
    where
        P: Into<mint::Point2<f32>> + Clone,
        V: t::StrokeVertexConstructor<Vertex> + t::FillVertexConstructor<Vertex>,
    {
        {
            assert!(points.len() > 1);
            let buffers = &mut self.buffer;
            let points: Vec<LPoint> = points
                .iter()
                .cloned()
                .map(|p| {
                    let mint_point: mint::Point2<f32> = p.into();
                    t::math::point(mint_point.x, mint_point.y)
                })
                .collect();
            let polygon = Polygon {
                points: &points,
                closed: is_closed,
            };
            match mode {
                DrawMode::Fill(options) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let tessellator = &mut t::FillTessellator::new();
                    let _ = tessellator.tessellate_polygon(polygon, &options, builder)?;
                }
                DrawMode::Stroke(options) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let tessellator = &mut t::StrokeTessellator::new();
                    let _ = tessellator.tessellate_polygon(polygon, &options, builder)?;
                }
            };
        }
        Ok(self)
    }

    /// Create a new mesh for a rectangle.
    pub fn rectangle(
        &mut self,
        mode: DrawMode,
        bounds: Rect,
        color: Color,
    ) -> GameResult<&mut Self> {
        {
            let buffers = &mut self.buffer;
            let rect = t::math::rect(bounds.x, bounds.y, bounds.w, bounds.h);
            let vb = VertexBuilder {
                color: LinearColor::from(color),
            };
            match mode {
                DrawMode::Fill(fill_options) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let mut tessellator = t::FillTessellator::new();
                    let _ = tessellator.tessellate_rectangle(&rect, &fill_options, builder);
                }
                DrawMode::Stroke(options) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let mut tessellator = t::StrokeTessellator::new();
                    let _ = tessellator.tessellate_rectangle(&rect, &options, builder);
                }
            };
        }
        Ok(self)
    }

    /// Create a new mesh for a rounded rectangle.
    pub fn rounded_rectangle(
        &mut self,
        mode: DrawMode,
        bounds: Rect,
        radius: f32,
        color: Color,
    ) -> GameResult<&mut Self> {
        {
            let buffers = &mut self.buffer;
            let rect = t::math::rect(bounds.x, bounds.y, bounds.w, bounds.h);
            let radii = t::path::builder::BorderRadii::new(radius);
            let vb = VertexBuilder {
                color: LinearColor::from(color),
            };
            let mut path_builder = t::path::Path::builder();
            path_builder.add_rounded_rectangle(&rect, &radii, t::path::Winding::Positive);
            let path = path_builder.build();

            match mode {
                DrawMode::Fill(fill_options) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let mut tessellator = t::FillTessellator::new();
                    let _ = tessellator.tessellate_path(&path, &fill_options, builder);
                }
                DrawMode::Stroke(options) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let mut tessellator = t::StrokeTessellator::new();
                    let _ = tessellator.tessellate_path(&path, &options, builder);
                }
            };
        }
        Ok(self)
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
                return Err(GameError::LyonError(String::from(
                    "Called Mesh::triangles() with points that have a length not a multiple of 3.",
                )));
            }
            let tris = triangles
                .iter()
                .cloned()
                .map(|p| {
                    // Gotta turn ggez Point2's into lyon points
                    let mint_point = p.into();
                    lyon::math::point(mint_point.x, mint_point.y)
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
            for tri in tris {
                // Ideally this assert makes bounds-checks only happen once.
                assert!(tri.len() == 3);
                let first_index: u32 = self.buffer.vertices.len().try_into().unwrap();
                self.buffer.vertices.push(vb.new_vertex(tri[0]));
                self.buffer.vertices.push(vb.new_vertex(tri[1]));
                self.buffer.vertices.push(vb.new_vertex(tri[2]));
                self.buffer.indices.push(first_index);
                self.buffer.indices.push(first_index + 1);
                self.buffer.indices.push(first_index + 2);
            }
        }
        Ok(self)
    }

    /// Takes an `Image` to apply to the mesh.
    pub fn texture(&mut self, texture: Image) -> GameResult<&mut Self> {
        self.image = Some(texture);
        Ok(self)
    }

    /// Creates a `Mesh` from a raw list of triangles defined from vertices
    /// and indices.  You may also
    /// supply an `Image` to use as a texture, if you pass `None`, it will
    /// just use a pure white texture.
    ///
    /// This is the most primitive mesh-creation method, but allows you full
    /// control over the tesselation and texturing.  It has the same constraints
    /// as `Mesh::from_raw()`.
    pub fn raw<V>(
        &mut self,
        verts: &[V],
        indices: &[u32],
        texture: Option<Image>,
    ) -> GameResult<&mut Self>
    where
        V: Into<Vertex> + Clone,
    {
        assert!(self.buffer.vertices.len() + verts.len() < (std::u32::MAX as usize));
        assert!(self.buffer.indices.len() + indices.len() < (std::u32::MAX as usize));
        let next_idx = self.buffer.vertices.len() as u32;
        // Can we remove the clone here?
        // I can't find a way to, because `into()` consumes its source and
        // `Borrow` or `AsRef` aren't really right.
        // EDIT: We can, but at a small cost to user-friendlyness, see:
        //       https://github.com/ggez/ggez/issues/940
        let vertices = verts.iter().cloned().map(|v: V| -> Vertex { v.into() });
        let indices = indices.iter().map(|i| (*i) + next_idx);
        self.buffer.vertices.extend(vertices);
        self.buffer.indices.extend(indices);
        self.image = texture;
        Ok(self)
    }

    /// Takes the accumulated geometry and load it into GPU memory,
    /// creating a single `Mesh`.
    ///
    /// Note that this returns a `GameResult<Mesh>`, since the build can fail,
    /// for example when trying to build an empty `MeshBuilder`.
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

impl VertexBuilder {
    fn new_vertex(self, position: LPoint) -> Vertex {
        Vertex {
            pos: [position.x, position.y],
            uv: [position.x, position.y],
            color: self.color.into(),
        }
    }
}

impl t::StrokeVertexConstructor<Vertex> for VertexBuilder {
    fn new_vertex(&mut self, vertex: t::StrokeVertex) -> Vertex {
        let position = vertex.position();
        Vertex {
            pos: [position.x, position.y],
            uv: [0.0, 0.0],
            color: self.color.into(),
        }
    }
}

impl t::FillVertexConstructor<Vertex> for VertexBuilder {
    fn new_vertex(&mut self, vertex: t::FillVertex) -> Vertex {
        let position = vertex.position();
        Vertex {
            pos: [position.x, position.y],
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

    /// Create a new mesh for a rounded rectangle
    pub fn new_rounded_rectangle(
        ctx: &mut Context,
        mode: DrawMode,
        bounds: Rect,
        radius: f32,
        color: Color,
    ) -> GameResult<Mesh> {
        let mut mb = MeshBuilder::new();
        let _ = mb.rounded_rectangle(mode, bounds, radius, color);
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
            let msg = String::from("Trying to build mesh with < 3 vertices, this is usually due to invalid input to a `Mesh` or MeshBuilder`.");
            return Err(GameError::LyonError(msg));
        }
        if indices.len() < 3 {
            let msg = format!("Trying to build mesh with < 3 indices, this is usually due to invalid input to a `Mesh` or MeshBuilder`.  Indices:\n {:#?}", indices);
            return Err(GameError::LyonError(msg));
        }

        if indices.len() % 3 != 0 {
            let msg = String::from("Trying to build mesh with an array of indices that is not a multiple of 3, this is usually due to invalid input to a `Mesh` or MeshBuilder`.");
            return Err(GameError::LyonError(msg));
        }

        let verts: Vec<Vertex> = verts.iter().cloned().map(Into::into).collect();
        let rect = bbox_for_vertices(&verts).expect(
            "No vertices in MeshBuilder; should never happen since we already checked this",
        );
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
        // also update the bounding box
        self.rect = bbox_for_vertices(verts).expect(
            "couldn't calculate bounding box; should never happen as long as `verts` isn't empty",
        );
    }

    /// Returns a slice for this mesh that could be used for manual draw call submission
    pub fn get_slice(&self) -> &gfx::Slice<gfx_device_gl::Resources> {
        &self.slice
    }

    /// Returns a vertex buffer for this mesh that could be used for manual draw call submission
    pub fn get_vertex_buffer(&self) -> gfx::handle::Buffer<gfx_device_gl::Resources, Vertex> {
        self.buffer.clone()
    }
}

impl Drawable for Mesh {
    fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult {
        self.debug_id.assert(ctx);
        let gfx = &mut ctx.gfx_context;
        gfx.update_instance_properties(param)?;

        gfx.data.vbuf = self.buffer.clone();
        let texture = self.image.texture.clone();
        let sampler = gfx
            .samplers
            .get_or_insert(self.image.sampler_info, gfx.factory.as_mut());

        let typed_thingy = gfx.backend_spec.raw_to_typed_shader_resource(texture);
        gfx.data.tex = (typed_thingy, sampler);
        let previous_mode: Option<BlendMode> = if let Some(mode) = self.blend_mode {
            let current_mode = gfx.blend_mode();
            if current_mode != mode {
                gfx.set_blend_mode(mode)?;
                Some(current_mode)
            } else {
                None
            }
        } else {
            None
        };

        gfx.draw(Some(&self.slice))?;
        if let Some(mode) = previous_mode {
            gfx.set_blend_mode(mode)?;
        }

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

/// An index of a particular instance in a `MeshBatch`
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MeshIdx(pub usize);

/// Mesh that will be rendered with hardware instancing.
/// Use this when you have a lot of similar geometry which does not move around often.
#[derive(Debug)]
pub struct MeshBatch {
    mesh: Mesh,
    instance_params: Vec<DrawParam>,
    instance_buffer: Option<gfx::handle::Buffer<gfx_device_gl::Resources, InstanceProperties>>,
    instance_buffer_dirty: bool,
}

impl MeshBatch {
    /// Creates a new mesh batch.
    ///
    /// Takes ownership of the `Mesh`.
    pub fn new(mesh: Mesh) -> GameResult<MeshBatch> {
        Ok(MeshBatch {
            mesh,
            instance_params: Vec::new(),
            instance_buffer: None,
            instance_buffer_dirty: true,
        })
    }

    /// Removes all instances from the batch.
    ///
    /// Calling this invalidates the entire buffer, however this will
    /// not automatically deallocate graphics card memory or flush the buffer.
    pub fn clear(&mut self) {
        self.instance_params.clear();
        self.instance_buffer_dirty = true;
    }

    /// Returns a reference to mesh instances
    pub fn get_instance_params(&self) -> &[DrawParam] {
        &self.instance_params
    }

    /// Returns a mutable reference to mesh instances.
    ///
    /// Please note that manually altering items in this slice
    /// will not automatically invalidate the buffer, you will
    /// have to manually call `flush()` or `flush_range()` later.
    pub fn get_instance_params_mut(&mut self) -> &mut [DrawParam] {
        &mut self.instance_params
    }

    /// Adds a new instance to the mesh batch
    ///
    /// Returns a handle with which to modify the instance using
    /// [`set()`](#method.set)
    ///
    /// Calling this invalidates the entire buffer and will result in
    /// flusing it on the next [`graphics::draw()`](../fn.draw.html) call.
    pub fn add<P>(&mut self, param: P) -> MeshIdx
    where
        P: Into<DrawParam>,
    {
        self.instance_params.push(param.into());
        self.instance_buffer_dirty = true;
        MeshIdx(self.instance_params.len() - 1)
    }

    /// Alters an instance in the batch to use the given draw params.
    ///
    /// Calling this invalidates the entire buffer and will result in
    /// flusing it on the next [`graphics::draw()`](../fn.draw.html) call.
    ///
    /// This might cause performance issues with large batches, to avoid this
    /// consider using `flush_range` to explicitly invalidate required data slice.
    pub fn set<P>(&mut self, handle: MeshIdx, param: P) -> GameResult
    where
        P: Into<DrawParam>,
    {
        if handle.0 < self.instance_params.len() {
            self.instance_params[handle.0] = param.into();
            self.instance_buffer_dirty = true;
            Ok(())
        } else {
            Err(GameError::RenderError(String::from("Index out of bounds")))
        }
    }

    /// Alters a range of instances in the batch to use the given draw params
    ///
    /// Calling this invalidates the entire buffer and will result in
    /// flusing it on the next [`graphics::draw()`](../fn.draw.html) call.
    ///
    /// This might cause performance issues with large batches, to avoid this
    /// consider using `flush_range` to explicitly invalidate required data slice.
    pub fn set_range<P>(&mut self, first_handle: MeshIdx, params: &[P]) -> GameResult
    where
        P: Into<DrawParam> + Copy,
    {
        let first_param = first_handle.0;
        let num_params = params.len();
        if first_param < self.instance_params.len()
            && (first_param + num_params) <= self.instance_params.len()
        {
            for (i, item) in params.iter().enumerate().take(num_params) {
                self.instance_params[first_param + i] = (*item).into();
            }
            self.instance_buffer_dirty = true;
            Ok(())
        } else {
            Err(GameError::RenderError(String::from("Range out of bounds")))
        }
    }

    /// Immediately sends specified slice of data in the batch to the graphics card.
    ///
    /// Calling this counts as a full buffer flush, but only flushes the data within
    /// the provided range, anything outside of this range will not be touched.
    ///
    /// Use it for updating small portions of large batches.
    pub fn flush_range(
        &mut self,
        ctx: &mut Context,
        first_handle: MeshIdx,
        count: usize,
    ) -> GameResult {
        let first_param = first_handle.0;
        let slice_len = first_param + count;
        if first_param < self.instance_params.len() && slice_len <= self.instance_params.len() {
            let needs_new_buffer = self.instance_buffer == None
                || self.instance_buffer.as_ref().unwrap().len() < slice_len;

            let slice = if needs_new_buffer {
                &self.instance_params
            } else {
                &self.instance_params[first_param..slice_len]
            };

            let new_properties: Vec<InstanceProperties> = slice
                .iter()
                .map(|param| param.to_instance_properties(ctx.gfx_context.is_srgb()))
                .collect();

            if needs_new_buffer {
                let new_buffer = ctx.gfx_context.factory.create_buffer(
                    new_properties.len(),
                    gfx::buffer::Role::Vertex,
                    gfx::memory::Usage::Dynamic,
                    gfx::memory::Bind::TRANSFER_DST,
                )?;

                self.instance_buffer = Some(new_buffer);

                ctx.gfx_context.encoder.update_buffer(
                    self.instance_buffer.as_ref().expect("Can never fail"),
                    new_properties.as_slice(),
                    0,
                )?;
            } else {
                ctx.gfx_context.encoder.update_buffer(
                    self.instance_buffer.as_ref().expect("Should never fail"),
                    new_properties.as_slice(),
                    first_param,
                )?;
            }

            self.instance_buffer_dirty = false;
            Ok(())
        } else {
            Err(GameError::RenderError(String::from("Range out of bounds")))
        }
    }

    /// Immediately sends all data in the batch to the graphics card.
    ///
    /// In general, [`graphics::draw()`](../fn.draw.html) on the `MeshBatch`
    /// will do this automatically when buffer contents are updated.
    pub fn flush(&mut self, ctx: &mut Context) -> GameResult {
        self.flush_range(ctx, MeshIdx(0), self.instance_params.len())
    }

    /// Draws the drawable onto the rendering target.
    pub fn draw(&mut self, ctx: &mut Context, param: DrawParam) -> GameResult {
        if !self.instance_params.is_empty() {
            self.mesh.debug_id.assert(ctx);

            if !self.instance_params.is_empty() && self.instance_buffer_dirty {
                self.flush(ctx)?;
            }

            let mut slice = self.mesh.slice.clone();
            slice.instances = Some((self.instance_params.len() as u32, 0));

            let gfx = &mut ctx.gfx_context;

            // In the batch we multiply the transform for each item in the batch
            // with the transform given in the `DrawParam` here.
            let batch_transform = Matrix4::from(param.trans.to_bare_matrix());
            gfx.set_global_mvp(batch_transform)?;

            // HACK this code has to restore the old instance buffer after drawing,
            // otherwise something else will override the first instance data
            let instance_buffer = self.instance_buffer.as_ref().expect("Should never fail");
            let old_instance_buffer = gfx.data.rect_instance_properties.clone();

            gfx.data.rect_instance_properties = instance_buffer.clone();
            gfx.data.vbuf = self.mesh.buffer.clone();
            let texture = self.mesh.image.texture.clone();
            let sampler = gfx
                .samplers
                .get_or_insert(self.mesh.image.sampler_info, gfx.factory.as_mut());

            let typed_thingy = gfx.backend_spec.raw_to_typed_shader_resource(texture);
            gfx.data.tex = (typed_thingy, sampler);

            gfx.draw(Some(&slice))?;

            gfx.data.rect_instance_properties = old_instance_buffer;

            // Undo the change we've made to the global MVP
            gfx.set_global_mvp(Matrix4::IDENTITY)?;
        }

        Ok(())
    }

    /// Returns a bounding box in the form of a `Rect`.
    pub fn dimensions(&self, ctx: &mut Context) -> Option<Rect> {
        if self.instance_params.is_empty() {
            return None;
        }
        if let Some(dimensions) = self.mesh.dimensions(ctx) {
            self.instance_params
                .iter()
                .map(|&param| transform_rect(dimensions, param))
                .fold(None, |acc: Option<Rect>, rect| {
                    Some(if let Some(acc) = acc {
                        acc.combine_with(rect)
                    } else {
                        rect
                    })
                })
        } else {
            None
        }
    }

    /// Sets the blend mode to be used when drawing this drawable.
    pub fn set_blend_mode(&mut self, mode: Option<BlendMode>) {
        self.mesh.set_blend_mode(mode)
    }

    /// Gets the blend mode to be used when drawing this drawable.
    pub fn blend_mode(&self) -> Option<BlendMode> {
        self.mesh.blend_mode()
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
