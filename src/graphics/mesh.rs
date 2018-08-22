use context::DebugId;
use gfx::traits::FactoryExt;
use graphics::*;
use lyon;
use lyon::tessellation as t;

pub use self::t::{FillOptions, FillRule, LineCap, LineJoin, StrokeOptions};

/// A builder for creating `Mesh`es.
///
/// This allows you to easily make one `Mesh` containing
/// many different complex pieces of geometry.  They don't
/// have to be connected to each other, and will all be
/// drawn at once.
///
/// The following example shows how to build a mesh containing a line and a circle:
///
/// ```rust,ignore
/// let mesh: Mesh = MeshBuilder::new()
///     .line(&[Point2::new(20.0, 20.0), Point2::new(40.0, 20.0)], 4.0)
///     .circle(DrawMode::Fill, Point2::new(60.0, 38.0), 40.0, 1.0)
///     .build(ctx)
///     .unwrap();
/// ```
/// A more sophisticated example:
///
/// ```rust
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
///         )
///         // Add vertices for an exclamation mark!
///         .ellipse(DrawMode::Fill, na::Point2::new(0.0, 25.0), 2.0, 15.0, 2.0)
///         .circle(DrawMode::Fill, na::Point2::new(0.0, 45.0), 2.0, 2.0)
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
    buffer: t::geometry_builder::VertexBuffers<Vertex, u16>,
}

impl Default for MeshBuilder {
    fn default() -> Self {
        Self {
            buffer: t::VertexBuffers::new(),
        }
    }
}

impl MeshBuilder {
    /// Create a new MeshBuilder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new mesh for a line of one or more connected segments.
    pub fn line<P>(&mut self, points: &[P], width: f32, color: Color) -> &mut Self
    where
        P: Into<mint::Point2<f32>> + Clone,
    {
        self.polyline(DrawMode::Line(width), points, color)
    }

    /// Create a new mesh for a circle.
    ///
    /// For the meaning of the `tolerance` parameter, [see here](https://docs.rs/lyon_geom/0.9.0/lyon_geom/#flattening).
    pub fn circle<P>(&mut self, mode: DrawMode, point: P, radius: f32, tolerance: f32, color: Color) -> &mut Self
    where
        P: Into<mint::Point2<f32>>,
    {
        {
            let point = point.into();
            let buffers = &mut self.buffer;
            let vb = VertexBuilder {
                color,
            };
            match mode {
                DrawMode::Fill => {
                    // These builders have to be in separate match arms 'cause they're actually
                    // different types; one is GeometryBuilder<StrokeVertex> and the other is
                    // GeometryBuilder<FillVertex>
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let fill_options = t::FillOptions::default().with_tolerance(tolerance);
                    let _ = t::basic_shapes::fill_circle(
                        t::math::point(point.x, point.y),
                        radius,
                        &fill_options,
                        builder,
                    );
                }
                DrawMode::Line(line_width) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let options = t::StrokeOptions::default()
                        .with_line_width(line_width)
                        .with_tolerance(tolerance);
                    let _ = t::basic_shapes::stroke_circle(
                        t::math::point(point.x, point.y),
                        radius,
                        &options,
                        builder,
                    );
                }
                DrawMode::CustomFill(fill_options) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let _ = t::basic_shapes::fill_circle(
                        t::math::point(point.x, point.y),
                        radius,
                        &fill_options.with_tolerance(tolerance),
                        builder,
                    );
                }
                DrawMode::CustomLine(options) => {
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
    /// For the meaning of the `tolerance` parameter, [see here](https://docs.rs/lyon_geom/0.9.0/lyon_geom/#flattening).
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
                color,
            };
            match mode {
                DrawMode::Fill => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let fill_options = t::FillOptions::default().with_tolerance(tolerance);
                    let _ = t::basic_shapes::fill_ellipse(
                        t::math::point(point.x, point.y),
                        t::math::vector(radius1, radius2),
                        t::math::Angle { radians: 0.0 },
                        &fill_options,
                        builder,
                    );
                }
                DrawMode::Line(line_width) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let options = t::StrokeOptions::default()
                        .with_line_width(line_width)
                        .with_tolerance(tolerance);
                    let _ = t::basic_shapes::stroke_ellipse(
                        t::math::point(point.x, point.y),
                        t::math::vector(radius1, radius2),
                        t::math::Angle { radians: 0.0 },
                        &options,
                        builder,
                    );
                }
                DrawMode::CustomFill(fill_options) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let _ = t::basic_shapes::fill_ellipse(
                        t::math::point(point.x, point.y),
                        t::math::vector(radius1, radius2),
                        t::math::Angle { radians: 0.0 },
                        &fill_options.with_tolerance(tolerance),
                        builder,
                    );
                }
                DrawMode::CustomLine(options) => {
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
    pub fn polyline<P>(&mut self, mode: DrawMode, points: &[P], color: Color) -> &mut Self
    where
        P: Into<mint::Point2<f32>> + Clone,
    {
        self.polyline_inner(mode, points, false, color)
    }

    /// Create a new mesh for a closed polygon.
    pub fn polygon<P>(&mut self, mode: DrawMode, points: &[P], color: Color) -> &mut Self
    where
        P: Into<mint::Point2<f32>> + Clone,
    {
        self.polyline_inner(mode, points, true, color)
    }

    fn polyline_inner<P>(&mut self, mode: DrawMode, points: &[P], is_closed: bool, color: Color) -> &mut Self
    where
        P: Into<mint::Point2<f32>> + Clone,
    {
        {
            assert!(points.len() > 1);
            let buffers = &mut self.buffer;
            let points = points.into_iter().cloned().map(|p| {
                let mint_point: mint::Point2<f32> = p.into();
                t::math::point(mint_point.x, mint_point.y)
            });
            let vb = VertexBuilder {
                color,
            };
            match mode {
                DrawMode::Fill => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let tessellator = &mut t::FillTessellator::new();
                    let options = t::FillOptions::default();
                    // TODO: Removing this expect would be rather nice.
                    let _ = t::basic_shapes::fill_polyline(points, tessellator, &options, builder)
                        .expect("Could not fill polygon?");
                }
                DrawMode::Line(width) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let options = t::StrokeOptions::default().with_line_width(width);
                    let _ = t::basic_shapes::stroke_polyline(points, is_closed, &options, builder);
                }
                DrawMode::CustomFill(options) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let tessellator = &mut t::FillTessellator::new();
                    // TODO: Removing this expect would be rather nice.
                    let _ = t::basic_shapes::fill_polyline(points, tessellator, &options, builder)
                        .expect("Could not fill polygon?");
                }
                DrawMode::CustomLine(options) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let _ = t::basic_shapes::stroke_polyline(points, is_closed, &options, builder);
                }
            };
        }
        self
    }

    /// Create a new mesh for a rectangle.
    pub fn rectangle(&mut self, mode: DrawMode, bounds: Rect, color: Color) -> &mut Self {
        {
            let buffers = &mut self.buffer;
            let rect = t::math::rect(bounds.x, bounds.y, bounds.w, bounds.h);
            let vb = VertexBuilder {
                color,
            };
            match mode {
                DrawMode::Fill => {
                    // These builders have to be in separate match arms 'cause they're actually
                    // different types; one is GeometryBuilder<StrokeVertex> and the other is
                    // GeometryBuilder<FillVertex>
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let fill_options = t::FillOptions::default();
                    let _ = t::basic_shapes::fill_rectangle(&rect, &fill_options, builder);
                }
                DrawMode::Line(line_width) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let options = t::StrokeOptions::default().with_line_width(line_width);
                    let _ = t::basic_shapes::stroke_rectangle(&rect, &options, builder);
                }
                DrawMode::CustomFill(fill_options) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let _ = t::basic_shapes::fill_rectangle(&rect, &fill_options, builder);
                }
                DrawMode::CustomLine(options) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, vb);
                    let _ = t::basic_shapes::stroke_rectangle(&rect, &options, builder);
                }
            };
        }
        self
    }

    /// Create a new `Mesh` from a raw list of triangles.
    ///
    /// Currently does not support UV's or indices.
    pub fn triangles<P>(&mut self, triangles: &[P], color: Color) -> &mut Self
    where
        P: Into<mint::Point2<f32>> + Clone,
    {
        {
            assert_eq!(triangles.len() % 3, 0);
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
                color,
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
                let i1 = builder.add_vertex(fst);
                let i2 = builder.add_vertex(snd);
                let i3 = builder.add_vertex(thd);
                builder.add_triangle(i1, i2, i3);
            }
            let _ = builder.end_geometry();
        }
        self
    }

    /// TODO: Update docs.
    /// Creates a `Mesh` from a raw list of triangles defined from points
    /// and indices, with the given UV texture coordinates.
    ///
    /// This is the most primitive mesh-creation method, but allows you full
    /// control over the tesselation and texturing.
    /// As such it will panic or produce incorrect/invalid output (that may later
    /// cause drawing to panic), if:
    ///
    ///  * `indices` contains a value out of bounds of `verts`
    pub fn from_raw<V>(&mut self, verts: &[V], indices: &[u16]) -> &mut Self
    where
        V: Into<Vertex> + Clone,
    {
        // TODO: Make `as` conversion checked
        let next_idx = self.buffer.vertices.len() as u16;
        // TODO: Can we remove the clone here?
        let vertices = verts.iter().cloned().map(|v: V| -> Vertex { v.into() });
        let indices = indices.iter().map(|i| (*i) + next_idx);
        self.buffer.vertices.extend(vertices);
        self.buffer.indices.extend(indices);
        self
    }

    /// Takes the accumulated geometry and load it into GPU memory,
    /// creating a single `Mesh`.
    pub fn build(&self, ctx: &mut Context) -> GameResult<Mesh> {
        let (vbuf, slice) = ctx.gfx_context
            .factory
            .create_vertex_buffer_with_slice(&self.buffer.vertices[..], &self.buffer.indices[..]);

        Ok(Mesh {
            buffer: vbuf,
            slice,
            blend_mode: None,
            debug_id: DebugId::get(ctx),
        })
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct VertexBuilder {
    color: Color,
}

impl t::VertexConstructor<t::FillVertex, Vertex> for VertexBuilder {
    fn new_vertex(&mut self, vertex: t::FillVertex) -> Vertex {
        Vertex {
            pos: [vertex.position.x, vertex.position.y],
            uv: [0.0, 0.0],
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
/// via a `MeshBuilder`.
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    buffer: gfx::handle::Buffer<gfx_device_gl::Resources, Vertex>,
    slice: gfx::Slice<gfx_device_gl::Resources>,
    blend_mode: Option<BlendMode>,
    debug_id: DebugId,
}

impl Mesh {
    /// Create a new mesh for a line of one or more connected segments.
    pub fn new_line<P>(ctx: &mut Context, points: &[P], width: f32, color: Color) -> GameResult<Mesh>
    where
        P: Into<mint::Point2<f32>> + Clone,
    {
        let mut mb = MeshBuilder::new();
        let _ = mb.polyline(DrawMode::Line(width), points, color);
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
    pub fn new_polyline<P>(ctx: &mut Context, mode: DrawMode, points: &[P], color: Color,) -> GameResult<Mesh>
    where
        P: Into<mint::Point2<f32>> + Clone,
    {
        let mut mb = MeshBuilder::new();
        let _ = mb.polyline(mode, points, color);
        mb.build(ctx)
    }

    /// Create a new mesh for closed polygon.
    pub fn new_polygon<P>(ctx: &mut Context, mode: DrawMode, points: &[P], color: Color,) -> GameResult<Mesh>
    where
        P: Into<mint::Point2<f32>> + Clone,
    {
        let mut mb = MeshBuilder::new();
        let _ = mb.polygon(mode, points, color);
        mb.build(ctx)
    }

    /// Create a new mesh for a rectangle
    pub fn new_rectangle(ctx: &mut Context, mode: DrawMode, bounds: Rect, color: Color,) -> GameResult<Mesh> {
        let mut mb = MeshBuilder::new();
        let _ = mb.rectangle(mode, bounds, color);
        mb.build(ctx)
    }

    /// Create a new `Mesh` from a raw list of triangle points.
    pub fn from_triangles<P>(ctx: &mut Context, triangles: &[P], color: Color,) -> GameResult<Mesh>
    where
        P: Into<mint::Point2<f32>> + Clone,
    {
        let mut mb = MeshBuilder::new();
        let _ = mb.triangles(triangles, color);
        mb.build(ctx)
    }

    /// Creates a `Mesh` from a raw list of triangles defined from points
    /// and indices, with the given UV texture coordinates.
    ///
    /// This is the most primitive mesh-creation method, but allows you full
    /// control over the tesselation and texturing.
    /// As such it will panic or produce incorrect/invalid output (that may later
    /// cause drawing to panic), if:
    ///
    ///  * `indices` contains a value out of bounds of `verts`
    pub fn from_raw<V>(ctx: &mut Context, verts: &[V], indices: &[u16]) -> Mesh
    where
        V: Into<Vertex> + Clone,
    {
        let verts: Vec<Vertex> = verts.iter().cloned().map(|v| v.into()).collect();
        let (vbuf, slice) = ctx.gfx_context
            .factory
            .create_vertex_buffer_with_slice(&verts[..], indices);
        Mesh {
            buffer: vbuf,
            slice,
            blend_mode: None,
            debug_id: DebugId::get(ctx),
        }
    }
}

impl Drawable for Mesh {
    fn draw<D>(&self, ctx: &mut Context, param: D) -> GameResult
    where
        D: Into<DrawTransform>,
    {
        let param = param.into();
        self.debug_id.assert(ctx);
        let gfx = &mut ctx.gfx_context;
        gfx.update_instance_properties(param)?;

        gfx.data.vbuf = self.buffer.clone();
        let texture = gfx.white_image.texture.clone();

        let typed_thingy = gfx.backend_spec.raw_to_typed_shader_resource(texture);
        gfx.data.tex.0 = typed_thingy;

        gfx.draw(Some(&self.slice))?;

        Ok(())
    }
    fn set_blend_mode(&mut self, mode: Option<BlendMode>) {
        self.blend_mode = mode;
    }
    fn blend_mode(&self) -> Option<BlendMode> {
        self.blend_mode
    }
}
