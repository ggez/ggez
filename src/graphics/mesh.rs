//!

use wgpu::util::DeviceExt;

use super::{context::GraphicsContext, gpu::arc::ArcBuffer};
use std::sync::atomic::AtomicUsize;

/// Vertex format uploaded to vertex buffers.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    /// `vec2` position.
    pub position: [f32; 2],
    /// `vec2` UV/texture coordinates.
    pub uv: [f32; 2],
}

impl Vertex {
    pub(crate) const fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] = [
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 8,
                shader_location: 1,
            },
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

static NEXT_MESH_ID: AtomicUsize = AtomicUsize::new(0);

/// Mesh data stored on the GPU as a vertex and index buffer.
#[derive(Debug)]
pub struct Mesh {
    pub(crate) verts: ArcBuffer,
    pub(crate) inds: ArcBuffer,
    pub(crate) verts_capacity: usize,
    pub(crate) inds_capacity: usize,
    pub(crate) vertex_count: usize,
    pub(crate) index_count: usize,
    pub(crate) id: usize,
}

impl Mesh {
    /// Create a new mesh from a list of vertices and indices.
    pub fn new(gfx: &GraphicsContext, vertices: &[Vertex], indices: &[u32]) -> Self {
        Mesh {
            verts: Self::create_verts(gfx, vertices),
            inds: Self::create_inds(gfx, indices),
            verts_capacity: vertices.len(),
            inds_capacity: indices.len(),
            vertex_count: vertices.len(),
            index_count: indices.len(),
            id: NEXT_MESH_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        }
    }

    /// Update the vertices of the mesh.
    #[allow(unsafe_code)]
    pub fn set_vertices(&mut self, gfx: &GraphicsContext, vertices: &[Vertex]) {
        self.vertex_count = vertices.len();
        if vertices.len() > self.verts_capacity {
            self.verts_capacity = vertices.len();
            self.verts = Self::create_verts(gfx, vertices);
            self.update_id();
        } else {
            gfx.queue.write_buffer(&self.verts, 0, unsafe {
                std::slice::from_raw_parts(
                    vertices as *const _ as *const u8,
                    vertices.len() * std::mem::size_of::<Vertex>(),
                )
            });
        }
    }

    /// Update the indices of the mesh.
    #[allow(unsafe_code)]
    pub fn set_indices(&mut self, gfx: &GraphicsContext, indices: &[u32]) {
        self.index_count = indices.len();
        if indices.len() > self.inds_capacity {
            self.inds_capacity = indices.len();
            self.inds = Self::create_inds(gfx, indices);
            self.update_id();
        } else {
            gfx.queue.write_buffer(&self.inds, 0, unsafe {
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
    fn create_verts(gfx: &GraphicsContext, vertices: &[Vertex]) -> ArcBuffer {
        ArcBuffer::new(
            gfx.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: unsafe {
                        std::slice::from_raw_parts(
                            vertices.as_ptr() as *const u8,
                            std::mem::size_of::<Vertex>() * vertices.len(),
                        )
                    },
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                }),
        )
    }

    #[allow(unsafe_code)]
    fn create_inds(gfx: &GraphicsContext, indices: &[u32]) -> ArcBuffer {
        ArcBuffer::new(
            gfx.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: unsafe {
                        std::slice::from_raw_parts(indices.as_ptr() as *const u8, 4 * indices.len())
                    },
                    usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                }),
        )
    }
}
