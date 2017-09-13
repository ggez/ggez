use *;
use graphics::*;
use lyon::tessellation as t;


/// A builder for creating `Mesh`es.
///
/// This allows you to easily make one `Mesh` containing
/// many different complex pieces of geometry.
pub struct MeshBuilder {
    buffer: t::geometry_builder::VertexBuffers<Vertex>,
}

impl MeshBuilder {

    pub fn new() -> Self {
        MeshBuilder {
            buffer: t::VertexBuffers::new(),
        }
    }

    /// Create a new mesh for a line of one or more connected segments.
    /// WIP, sorry
    pub fn line(&mut self, points: &[Point], width: f32) -> &mut Self {
        self.polyline(DrawMode::Line(width), points)
    }

    /// Create a new mesh for a circle.
    /// Stroked circles are still WIP, sorry.
    pub fn circle(&mut self,
                      mode: DrawMode,
                      point: Point,
                      radius: f32,
                      tolerance: f32)
                      -> &mut Self {
        {
            let buffers = &mut self.buffer;
            match mode {
                DrawMode::Fill => {
                    // These builders have to be in separate match arms 'cause they're actually
                    // different types; one is GeometryBuilder<StrokeVertex> and the other is
                    // GeometryBuilder<FillVertex>
                    let builder = &mut t::BuffersBuilder::new(buffers, VertexBuilder);
                    t::basic_shapes::fill_circle(t::math::point(point.x, point.y),
                                                 radius,
                                                 tolerance,
                                                 builder);
                }
                DrawMode::Line(line_width) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, VertexBuilder);
                    let options = t::StrokeOptions::default()
                        .with_line_width(line_width)
                        .with_tolerance(tolerance);
                    t::basic_shapes::stroke_circle(t::math::point(point.x, point.y),
                                                   radius,
                                                   &options,
                                                   builder);
                }
            };
        }
        self

    }

    /// Create a new mesh for an ellipse.
    /// Stroked ellipses are still WIP, sorry.
    pub fn ellipse(&mut self,
                       mode: DrawMode,
                       point: Point,
                       radius1: f32,
                       radius2: f32,
                       tolerance: f32)
                       -> &mut Self {
        {
            let buffers = &mut self.buffer;
            use euclid::Length;
            match mode {
                DrawMode::Fill => {
                    // These builders have to be in separate match arms 'cause they're actually
                    // different types; one is GeometryBuilder<StrokeVertex> and the other is
                    // GeometryBuilder<FillVertex>
                    let builder = &mut t::BuffersBuilder::new(buffers, VertexBuilder);
                    t::basic_shapes::fill_ellipse(t::math::point(point.x, point.y),
                                                  t::math::vec2(radius1, radius2),
                                                  Length::new(0.0),
                                                  tolerance,
                                                  builder);
                }
                DrawMode::Line(line_width) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, VertexBuilder);
                    let options = t::StrokeOptions::default()
                        .with_line_width(line_width)
                        .with_tolerance(tolerance);
                    t::basic_shapes::stroke_ellipse(t::math::point(point.x, point.y),
                                                    t::math::vec2(radius1, radius2),
                                                    Length::new(0.0),
                                                    &options,
                                                    builder);
                }
            };
        }
        self
    }

    /// Create a new mesh for series of connected lines
    pub fn polyline(&mut self, mode: DrawMode, points: &[Point]) -> &mut Self {
        {
            let buffers = &mut self.buffer;
            let points = points
                .into_iter()
                .map(|ggezpoint| t::math::point(ggezpoint.x, ggezpoint.y));
            match mode {
                DrawMode::Fill => {
                    // These builders have to be in separate match arms 'cause they're actually
                    // different types; one is GeometryBuilder<StrokeVertex> and the other is
                    // GeometryBuilder<FillVertex>
                    let builder = &mut t::BuffersBuilder::new(buffers, VertexBuilder);
                    let tessellator = &mut t::FillTessellator::new();
                    let options = t::FillOptions::default();
                    t::basic_shapes::fill_polyline(points, tessellator, &options, builder).unwrap();
                }
                DrawMode::Line(width) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, VertexBuilder);
                    let options = t::StrokeOptions::default().with_line_width(width);
                    t::basic_shapes::stroke_polyline(points, false, &options, builder);
                }
            };
        }
        self
    }

    /// Create a new mesh for closed polygon
    pub fn polygon(&mut self, mode: DrawMode, points: &[Point]) -> &mut Self {
        {
            let buffers = &mut self.buffer;
            let points = points
                .into_iter()
                .map(|ggezpoint| t::math::point(ggezpoint.x, ggezpoint.y));
            match mode {
                DrawMode::Fill => {
                    // These builders have to be in separate match arms 'cause they're actually
                    // different types; one is GeometryBuilder<StrokeVertex> and the other is
                    // GeometryBuilder<FillVertex>
                    let builder = &mut t::BuffersBuilder::new(buffers, VertexBuilder);
                    let tessellator = &mut t::FillTessellator::new();
                    let options = t::FillOptions::default();
                    t::basic_shapes::fill_polyline(points, tessellator, &options, builder).unwrap();
                }
                DrawMode::Line(width) => {
                    let builder = &mut t::BuffersBuilder::new(buffers, VertexBuilder);
                    let options = t::StrokeOptions::default().with_line_width(width);
                    t::basic_shapes::stroke_polyline(points, true, &options, builder);
                }
            };
        }
        self
    }

    /// BUGGO: TODO
    /// Create a new `Mesh` from a raw list of triangles.
    ///
    /// Currently does not support UV's or indices.
    // pub fn from_triangles(&mut self, triangles: &[Point]) -> &mut Self {
    //     // This is kind of non-ideal but works for now.
    //     let points = triangles
    //         .into_iter()
    //         .map(|p| {
    //                  Vertex {
    //                      pos: (*p).into(),
    //                      uv: (*p).into(),
    //                  }
    //              });
    //     self.buffer.extend(points);
    //     self
    // }

    pub fn build(&self, ctx: &mut Context) -> GameResult<Mesh> {
        let (vbuf, slice) =
            ctx.gfx_context
                .factory
                .create_vertex_buffer_with_slice(&self.buffer.vertices[..],
                                                 &self.buffer.indices[..]);

        Ok(Mesh {
               buffer: vbuf,
               slice: slice,
           })
    }
}

// Lyon's VertexBuffers doesn't impl debug or clone, see https://github.com/nical/lyon/issues/167
impl fmt::Debug for MeshBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<MeshBuilder>")
    }
}


struct VertexBuilder;

impl t::VertexConstructor<t::FillVertex, Vertex> for VertexBuilder {
    fn new_vertex(&mut self, vertex: t::FillVertex) -> Vertex {
        Vertex {
            pos: [vertex.position.x, vertex.position.y],
            uv: [0.0, 0.0],
        }
    }
}

impl t::VertexConstructor<t::StrokeVertex, Vertex> for VertexBuilder {
    fn new_vertex(&mut self, vertex: t::StrokeVertex) -> Vertex {
        Vertex {
            pos: [vertex.position.x, vertex.position.y],
            uv: [0.0, 0.0],
        }
    }
}


/// 2D polygon mesh
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    buffer: gfx::handle::Buffer<gfx_device_gl::Resources, Vertex>,
    slice: gfx::Slice<gfx_device_gl::Resources>,
}


impl Mesh {
    fn from_vbuf(ctx: &mut Context,
                 buffer: &t::geometry_builder::VertexBuffers<Vertex>)
                 -> GameResult<Mesh> {
        let (vbuf, slice) =
            ctx.gfx_context
                .factory
                .create_vertex_buffer_with_slice(&buffer.vertices[..], &buffer.indices[..]);

        Ok(Mesh {
               buffer: vbuf,
               slice: slice,
           })
    }


    /// Create a new mesh for a line of one or more connected segments.
    /// WIP, sorry
    pub fn new_line(ctx: &mut Context, points: &[Point], width: f32) -> GameResult<Mesh> {
        let mut mb = MeshBuilder::new();
        mb.polyline(DrawMode::Line(width), points);
        mb.build(ctx)
    }

    /// Create a new mesh for a circle.
    /// Stroked circles are still WIP, sorry.
    pub fn new_circle(ctx: &mut Context,
                      mode: DrawMode,
                      point: Point,
                      radius: f32,
                      tolerance: f32)
                      -> GameResult<Mesh> {
        let mut mb = MeshBuilder::new();
        mb.circle(mode, point, radius, tolerance);
        mb.build(ctx)
    }

    /// Create a new mesh for an ellipse.
    /// Stroked ellipses are still WIP, sorry.
    pub fn new_ellipse(ctx: &mut Context,
                       mode: DrawMode,
                       point: Point,
                       radius1: f32,
                       radius2: f32,
                       tolerance: f32)
                       -> GameResult<Mesh> {
        let mut mb = MeshBuilder::new();
        mb.ellipse(mode, point, radius1, radius2, tolerance);
        mb.build(ctx)
    }

    /// Create a new mesh for series of connected lines
    pub fn new_polyline(ctx: &mut Context,
                        mode: DrawMode,
                        points: &[Point])
                        -> GameResult<Mesh> {
        let mut mb = MeshBuilder::new();
        mb.polyline(mode, points);
        mb.build(ctx)
    }


    /// Create a new mesh for closed polygon
    pub fn new_polygon(ctx: &mut Context,
                       mode: DrawMode,
                       points: &[Point])
                       -> GameResult<Mesh> {
        let mut mb = MeshBuilder::new();
        mb.polygon(mode, points);
        mb.build(ctx)
    }

    /// Create a new `Mesh` from a raw list of triangles.
    ///
    /// Currently does not support UV's or indices.
    pub fn from_triangles(ctx: &mut Context, triangles: &[Point]) -> GameResult<Mesh> {
        // This is kind of non-ideal but works for now.
        let points: Vec<Vertex> = triangles
            .into_iter()
            .map(|p| {
                     Vertex {
                         pos: (*p).into(),
                         uv: (*p).into(),
                     }
                 })
            .collect();
        let (vbuf, slice) = ctx.gfx_context
            .factory
            .create_vertex_buffer_with_slice(&points[..], ());

        Ok(Mesh {
               buffer: vbuf,
               slice: slice,
           })
    }
}

impl Drawable for Mesh {
    fn draw_ex(&self, ctx: &mut Context, param: DrawParam) -> GameResult<()> {
        let gfx = &mut ctx.gfx_context;
        gfx.update_rect_properties(param)?;

        gfx.data.vbuf = self.buffer.clone();
        gfx.data.tex.0 = gfx.white_image.texture.clone();

        gfx.encoder.draw(&self.slice, &gfx.pso, &gfx.data);

        Ok(())
    }
}