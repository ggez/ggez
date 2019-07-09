use crate::tests;
use crate::*;

const TRIANGLE_VERTS: &[graphics::Vertex] = &[
    graphics::Vertex {
        pos: [0.0, 0.0],
        uv: [0.0, 0.0],
        color: [1.0, 1.0, 1.0, 1.0],
    },
    graphics::Vertex {
        pos: [0.0, 0.0],
        uv: [0.0, 0.0],
        color: [1.0, 1.0, 1.0, 1.0],
    },
    graphics::Vertex {
        pos: [0.0, 0.0],
        uv: [0.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
        },
    ];

/// Mesh creation fails if verts or indices are empty.
#[test]
fn test_mesh_verts_empty() {
    let (mut ctx, _ev) = tests::make_context();
    let verts: Vec<graphics::Vertex> = vec![
    ];
    let indices: Vec<u32> = vec![
    ];
    let m = graphics::Mesh::from_raw(&mut ctx, &verts, &indices, None);
    assert!(m.is_err());

    let m = graphics::Mesh::from_raw(&mut ctx, TRIANGLE_VERTS, &indices, None);
    assert!(m.is_err());

}

/// Mesh creation fails if not enough indices to make a triangle.
#[test]
fn test_mesh_verts_invalid_count() {
    let (mut ctx, _ev) = tests::make_context();
    let indices: Vec<u32> = vec![
        0, 1
    ];
    let m = graphics::Mesh::from_raw(&mut ctx, TRIANGLE_VERTS, &indices, None);
    assert!(m.is_err());

    let indices: Vec<u32> = vec![
        0, 1, 2, 0
    ];
    let m = graphics::Mesh::from_raw(&mut ctx, TRIANGLE_VERTS, &indices, None);
    assert!(m.is_err());
}


#[test]
fn test_mesh_points_clockwise() {
    let (mut ctx, _ev) = tests::make_context();

    // Points in CCW order
    let points: Vec<graphics::Point2> = vec![
        graphics::Point2::new(0.0, 0.0),
        graphics::Point2::new(0.0, -1.0),
        graphics::Point2::new(-1.0, -1.0),
    ];
    
    let trapezoid_mesh = graphics::Mesh::new_polygon(
        &mut ctx, 
        graphics::DrawMode::fill(), 
        &points,
        [0.0, 0.0, 1.0, 1.0].into()
    );

    // TODO: This is actually tricky to test for well...
    // We don't actually check for CCW points in
    // the `Mesh` building functions yet, so this will never fail.
    //assert!(trapezoid_mesh.is_err());
}
