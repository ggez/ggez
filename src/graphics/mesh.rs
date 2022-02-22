//!

use super::util;
use crate::context::Context;
use std::sync::atomic::AtomicUsize;
use wgpu::util::DeviceExt;

/// Vertex format uploaded to vertex buffers.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    /// `vec2` position.
    pub position: [f32; 2],
    /// `vec2` UV/texture coordinates.
    pub uv: [f32; 2],
}

static NEXT_MESH_ID: AtomicUsize = AtomicUsize::new(0);

/// Mesh data stored on the GPU as a vertex and index buffer.
#[derive(Debug)]
pub struct Mesh {
    pub(crate) verts: wgpu::Buffer,
    pub(crate) inds: wgpu::Buffer,
    pub(crate) verts_capacity: usize,
    pub(crate) inds_capacity: usize,
    pub(crate) vertex_count: usize,
    pub(crate) index_count: usize,
    pub(crate) id: usize,
}

impl Mesh {
    /// Create a new mesh from a list of vertices and indices.
    pub fn new(ctx: &Context, vertices: &[Vertex], indices: &[u32]) -> Self {
        Mesh {
            verts: Self::create_verts(ctx, vertices),
            inds: Self::create_inds(ctx, indices),
            verts_capacity: vertices.len(),
            inds_capacity: indices.len(),
            vertex_count: vertices.len(),
            index_count: indices.len(),
            id: NEXT_MESH_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        }
    }

    /// Update the vertices of the mesh.
    #[allow(unsafe_code)]
    pub fn set_vertices(&mut self, ctx: &Context, vertices: &[Vertex]) {
        self.vertex_count = vertices.len();
        if vertices.len() > self.verts_capacity {
            self.verts_capacity = vertices.len();
            self.verts = Self::create_verts(ctx, vertices);
            self.update_id();
        } else {
            ctx.gfx_context.queue.write_buffer(&self.verts, 0, unsafe {
                std::slice::from_raw_parts(
                    vertices as *const _ as *const u8,
                    vertices.len() * std::mem::size_of::<Vertex>(),
                )
            });
        }
    }

    /// Update the indices of the mesh.
    #[allow(unsafe_code)]
    pub fn set_indices(&mut self, ctx: &Context, indices: &[u32]) {
        self.index_count = indices.len();
        if indices.len() > self.inds_capacity {
            self.inds_capacity = indices.len();
            self.inds = Self::create_inds(ctx, indices);
            self.update_id();
        } else {
            ctx.gfx_context.queue.write_buffer(&self.inds, 0, unsafe {
                std::slice::from_raw_parts(
                    indices as *const _ as *const u8,
                    indices.len() * std::mem::size_of::<u32>(),
                )
            });
        }
    }

    fn update_id(&mut self) {
        self.id = NEXT_MESH_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    #[allow(unsafe_code)]
    fn create_verts(ctx: &Context, vertices: &[Vertex]) -> wgpu::Buffer {
        util::create_buffer_init_defer(
            &ctx.gfx_context,
            &util::BufferInitDeferDescriptor {
                data: unsafe { util::slice_to_bytes(vertices) },
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            },
        )
    }

    #[allow(unsafe_code)]
    fn create_inds(ctx: &Context, indices: &[u32]) -> wgpu::Buffer {
        util::create_buffer_init_defer(
            &ctx.gfx_context,
            &util::BufferInitDeferDescriptor {
                data: unsafe { util::slice_to_bytes(indices) },
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            },
        )
    }
}
