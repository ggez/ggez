use lyon::path;
use lyon::path_builder::BaseBuilder;
use lyon::path_iterator::PathIterator;
use lyon::tessellation;
use lyon::tessellation::basic_shapes;
use lyon::tessellation::math;
use lyon::tessellation::path_stroke;
use lyon::tessellation::path_fill;
use lyon::tessellation::geometry_builder;

use super::{Point, Vertex};
use GameError;
use GameResult;

pub type Buffer = geometry_builder::VertexBuffers<Vertex>;

const FLATTEN_TOLERANCE: f32 = 0.5;

pub struct VertexConstructor {
    stroke_width: f32,
}

impl geometry_builder::VertexConstructor<tessellation::StrokeVertex, Vertex> for VertexConstructor {
    fn new_vertex(&mut self, input: tessellation::StrokeVertex) -> Vertex {
        let c = input.position + input.normal * self.stroke_width;
        Vertex {
            pos: [c.x, c.y],
            uv: [c.x, c.y],
        }
    }
}

impl geometry_builder::VertexConstructor<tessellation::FillVertex, Vertex> for VertexConstructor {
    fn new_vertex(&mut self, input: tessellation::FillVertex) -> Vertex {
        let p = input.position;
        Vertex {
            pos: [p.x, p.y],
            uv: [p.x, p.y],
        }
    }
}


fn build_path(points: &[Point], closed: bool) -> path::Path {
    let mut path_builder = path::Builder::with_capacity(points.len());
    path_builder.move_to(math::point(points[0].x, points[0].y));

    for p in &points[1..] {
        path_builder.line_to(math::point(p.x, p.y));
    }

    if closed {
        path_builder.close();
    }

    path_builder.build()
}

fn build_geometry<F, V, E>(line_width: f32, f: F) -> GameResult<Buffer>
    where F: for<'a> FnOnce(&mut geometry_builder::BuffersBuilder<'a, Vertex, V, VertexConstructor>)
                            -> Result<geometry_builder::Count, E>,
        VertexConstructor: geometry_builder::VertexConstructor<V, Vertex>,
{
    let mut buffers = geometry_builder::VertexBuffers::new();
    let vertex_ctor = VertexConstructor { stroke_width: line_width };
    {
        let mut builder = geometry_builder::BuffersBuilder::new(&mut buffers, vertex_ctor);
        if let Err(_) = f(&mut builder) {
            return Err(GameError::RenderError(String::from("geometry tessellation failed")));
        }
    }
    Ok(buffers)
}

fn build_stroke(points: &[Point], close: bool, line_width: f32) -> GameResult<Buffer> {
    let path = build_path(points, close);
    let path_iter = path.path_iter().flattened(FLATTEN_TOLERANCE);
    let opts = path_stroke::StrokeOptions::default();
    let mut tessellator = path_stroke::StrokeTessellator::new();
    build_geometry(line_width, |builder| tessellator.tessellate(path_iter, &opts, builder))
}

pub fn build_line(points: &[Point], line_width: f32) -> GameResult<Buffer> {
    build_stroke(points, false, line_width)
}

/// Build a closed polygon.  Identical to build_line but closes the path,
/// which makes sure the two endpoints actually line up.
pub fn build_polygon(points: &[Point], line_width: f32) -> GameResult<Buffer> {
    build_stroke(points, true, line_width)
}

 pub fn build_polygon_fill(points: &[Point]) -> GameResult<Buffer> {
     let path = build_path(points, true);
     let path_iter = path.path_iter().flattened(FLATTEN_TOLERANCE);
     let opts = path_fill::FillOptions::default();
     let mut tessellator = path_fill::FillTessellator::new();
     build_geometry(0.0, |builder| tessellator.tessellate_path(path_iter, &opts, builder))
 }

pub fn build_ellipse_fill(point: Point, r1: f32, r2: f32, segments: u32) -> GameResult<Buffer> {
    let center = math::point(point.x, point.y);
    let radii = math::point(r1, r2);
    build_geometry(0.0, |builder| {
        let count = basic_shapes::fill_ellipse(center, radii, segments, builder);
        Ok::<_, ()>(count)
    })
}
