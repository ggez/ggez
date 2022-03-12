use super::{
    arc::{ArcTexture, ArcTextureView},
    growing::GrowingBufferArena,
};
use crate::{graphics::context::FrameArenas, GameResult};
use glyph_brush::{GlyphBrush, GlyphBrushBuilder};
use std::num::NonZeroU32;

pub struct TextRenderer {
    pub glyph_brush: GlyphBrush<TextVertex>,
    pub cache: ArcTexture,
    pub cache_view: ArcTextureView,
    pub cache_size: (u32, u32),
    pub verts: GrowingBufferArena,
}

impl TextRenderer {
    pub fn new(device: &wgpu::Device) -> Self {
        let cache_size = (1024, 1024);

        let glyph_brush = GlyphBrushBuilder::using_fonts(vec![])
            .cache_redraws(false)
            .initial_cache_size(cache_size)
            .build();

        let cache = ArcTexture::new(device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: cache_size.0,
                height: cache_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        }));

        let cache_view =
            ArcTextureView::new(cache.create_view(&wgpu::TextureViewDescriptor::default()));

        let verts = GrowingBufferArena::new(
            device,
            1,
            wgpu::BufferDescriptor {
                label: None,
                size: 2048 * std::mem::size_of::<TextVertex>() as u64,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            },
        );

        TextRenderer {
            glyph_brush,
            cache,
            cache_view,
            cache_size,
            verts,
        }
    }

    pub fn queue(&mut self, section: glyph_brush::Section<'_>) {
        self.glyph_brush.queue(section);
    }

    #[allow(unsafe_code)]
    pub(crate) fn draw_queued<'a>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        arenas: &'a FrameArenas,
        pass: &mut wgpu::RenderPass<'a>,
    ) -> GameResult<()> {
        let res = self.glyph_brush.process_queued(
            |rect, pixels| {
                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &self.cache,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: rect.min[0],
                            y: rect.min[1],
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    pixels,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(NonZeroU32::new(rect.width()).unwrap()),
                        rows_per_image: None,
                    },
                    wgpu::Extent3d {
                        width: rect.width(),
                        height: rect.height(),
                        depth_or_array_layers: 1,
                    },
                );
            },
            |glyph| TextVertex {
                rect: [
                    glyph.pixel_coords.min.x,
                    glyph.pixel_coords.min.y,
                    glyph.pixel_coords.max.x,
                    glyph.pixel_coords.max.y,
                ],
                uv: [
                    glyph.tex_coords.min.x,
                    glyph.tex_coords.min.y,
                    glyph.tex_coords.max.x,
                    glyph.tex_coords.max.y,
                ],
                color: glyph.extra.color,
            },
        );

        match res {
            Ok(glyph_brush::BrushAction::Draw(verts)) => {
                let verts_size = verts.len() * std::mem::size_of::<TextVertex>();
                let verts_alloc = self.verts.allocate(device, verts_size as u64);

                queue.write_buffer(&verts_alloc.buffer, verts_alloc.offset, unsafe {
                    std::slice::from_raw_parts(verts.as_ptr() as *const u8, verts_size)
                });

                let verts_buf = arenas.buffers.alloc(verts_alloc.buffer);
                pass.set_vertex_buffer(0, verts_buf.slice(verts_alloc.offset..));
                pass.draw(0..4, 0..verts.len() as u32);

                Ok(())
            }
            Err(glyph_brush::BrushError::TextureTooSmall { suggested }) => {
                self.cache_size = suggested;
                self.glyph_brush
                    .resize_texture(self.cache_size.0, self.cache_size.1);

                self.cache = ArcTexture::new(device.create_texture(&wgpu::TextureDescriptor {
                    label: None,
                    size: wgpu::Extent3d {
                        width: self.cache_size.0,
                        height: self.cache_size.1,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::R8Unorm,
                    usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
                }));

                self.cache_view = ArcTextureView::new(
                    self.cache
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                );

                self.draw_queued(device, queue, arenas, pass)
            }
            _ => unreachable!(),
        }
    }

    pub fn free(&mut self) {
        self.verts.free();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct TextVertex {
    pub rect: [f32; 4],
    pub uv: [f32; 4],
    pub color: [f32; 4],
}

impl TextVertex {
    pub(crate) const fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 3] = [
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 16,
                shader_location: 1,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 32,
                shader_location: 2,
            },
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TextVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRIBUTES,
        }
    }
}
