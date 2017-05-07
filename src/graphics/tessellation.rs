use lyon::path;
use lyon::path_builder::BaseBuilder;
use lyon::path_iterator::PathIterator;
use lyon::tessellation;
use lyon::tessellation::math;
use lyon::tessellation::path_fill;
use lyon::tessellation::path_stroke;
use lyon::tessellation::geometry_builder;

use super::{Point, Vertex};
use GameError;
use GameResult;

pub type Buffer = geometry_builder::VertexBuffers<Vertex>;

// Not used anywhere?  Not sure what vickenty was planning for this.
// pub struct ConstantUV {
//     uv: [f32; 2],
// }

// impl geometry_builder::VertexConstructor<math::Point, Vertex> for ConstantUV {
//     fn new_vertex(&mut self, input: math::Point) -> Vertex {
//         Vertex {
//             pos: [input.x, input.y],
//             uv: self.uv.clone(),
//         }
//     }
// }

pub struct ScreenUV;

impl geometry_builder::VertexConstructor<math::Point, Vertex> for ScreenUV {
    fn new_vertex(&mut self, input: math::Point) -> Vertex {
        Vertex {
            pos: [input.x, input.y],
            uv: [input.x, input.y],
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

type BuffersBuilder<'a> = geometry_builder::BuffersBuilder<'a, Vertex, math::Point, ScreenUV>;

fn build_geometry<F>(f: F) -> GameResult<Buffer>
    where F: for<'a> FnOnce(&mut BuffersBuilder<'a>)
                            -> Result<tessellation::geometry_builder::Count, ()>
{
    let mut buffers = geometry_builder::VertexBuffers::new();
    {
        let mut builder = geometry_builder::BuffersBuilder::new(&mut buffers, ScreenUV);
        if let Err(()) = f(&mut builder) {
            return Err(GameError::RenderError(String::from("geometry tessellation failed")));
        }
    }
    Ok(buffers)
}

pub fn build_line(points: &[Point], line_width: f32) -> GameResult<Buffer> {
    let path = build_path(points, false);
    let opts = path_stroke::StrokeOptions::stroke_width(line_width);
    let mut tessellator = path_stroke::StrokeTessellator::new();
    build_geometry(|builder| {
        tessellator.tessellate(path.path_iter()
                                   .flattened(0.5),
                               &opts,
                               builder)
    })
}

/// Build a closed polygon.  Identical to build_line but closes the path,
/// which makes sure the two endpoints actually line up.
pub fn build_polygon(points: &[Point], line_width: f32) -> GameResult<Buffer> {
    
    let path = build_path(points, true);
    let opts = path_stroke::StrokeOptions::stroke_width(line_width);
    let mut tessellator = path_stroke::StrokeTessellator::new();
    build_geometry(|builder| {
        tessellator.tessellate(path.path_iter()
                                   .flattened(0.5),
                               &opts,
                               builder)
    })
}

// pub fn build_polygon_fill(points: &[Point]) -> GameResult<Buffer> {
    
//     let path = build_path(points, true);
//     let opts = path_fill::FillOptions::default();
//     let mut tessellator = path_fill::FillTessellator::new();
//     build_geometry(|builder| {
//         tessellator.tessellate_events(path.path_iter()
//                                    .flattened(0.5),
//                                &opts,
//                                builder)
//     })
// }

pub fn build_ellipse_fill(point: Point, r1: f32, r2: f32, segments: u32) -> GameResult<Buffer> {
    build_geometry(|builder| {
        Ok(tessellation::basic_shapes::tessellate_ellipsis(math::point(point.x, point.y),
                                                           math::point(r1, r2),
                                                           segments,
                                                           builder))
    })
}
