use lyon;
use lyon::path;
use lyon::path_builder::BaseBuilder;
use lyon::path_iterator::PathIterator;
use lyon::tessellation;
use lyon::tessellation::math;
use lyon::tessellation::path_stroke;
use lyon::tessellation::geometry_builder;

use super::{ Point, Vertex };

pub type Buffer = geometry_builder::VertexBuffers<Vertex>;

pub struct ConstantUV {
    uv: [f32; 2],
}

impl geometry_builder::VertexConstructor<math::Point, Vertex> for ConstantUV {
    fn new_vertex(&mut self, input: math::Point) -> Vertex {
        Vertex {
            pos: [ input.x, input.y ],
            uv: self.uv.clone(),
        }
    }
}

pub struct ScreenUV;

impl geometry_builder::VertexConstructor<math::Point, Vertex> for ScreenUV {
    fn new_vertex(&mut self, input: math::Point) -> Vertex {
        Vertex {
            pos: [ input.x, input.y ],
            uv: [ input.x, input.y ],
        }
    }
}

fn build_path(points: &[Point]) -> path::Path {
    let mut path_builder = path::Builder::with_capacity(points.len());
    path_builder.move_to(math::point(points[0].x, points[0].y));

    for p in &points[1..] {
        path_builder.line_to(math::point(p.x, p.y));
    }

    path_builder.build()
}

pub fn build_line<T>(points: &[Point], line_width: f32, ctor: T) -> Result<Buffer, ()>
    where T: geometry_builder::VertexConstructor<math::Point, Vertex>
{
    let path = build_path(points);

    let mut buffers = geometry_builder::VertexBuffers::new();
    {
        let mut buf_builder = geometry_builder::BuffersBuilder::new(&mut buffers, ctor);

        let opts = path_stroke::StrokeOptions::stroke_width(line_width);
        let mut tesselator = path_stroke::StrokeTessellator::new();
        tesselator.tessellate(path.path_iter().flattened(0.5), &opts, &mut buf_builder)?;
    }

    Ok(buffers)
}
