use crate::painters::rect::RectPainter;
use painting::{Color, RRect, Rect};
use super::backend::{DrawRequest, Backend};
use super::Bitmap;

pub struct Painter<'a> {
    rect_painter: RectPainter,
    backend: Backend,
    device: wgpu::Device,
    queue: wgpu::Queue,
    frame_desc: wgpu::TextureDescriptor<'a>,
    frame: wgpu::Texture,
    frame_texture_view: wgpu::TextureView,
    output_buffer: wgpu::Buffer,
    output_buffer_desc: wgpu::BufferDescriptor<'a>,
}

pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

impl<'a> Painter<'a> {
    pub async fn new() -> Painter<'a> {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
        })
        .await
        .unwrap();

        let (device, queue) = adapter
            .request_device(&Default::default(), None)
            .await
            .unwrap();

        let frame_desc = wgpu::TextureDescriptor {
            label: Some("moon output texture"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TEXTURE_FORMAT,
            usage: wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::RENDER_ATTACHMENT,
        };

        let frame = device.create_texture(&frame_desc);

        let frame_texture_view = frame.create_view(&Default::default());
        let output_buffer_desc = wgpu::BufferDescriptor {
            label: Some("moon output buffer"),
            size: 1,
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::MAP_READ,
            mapped_at_creation: false,
        };
        let output_buffer = device.create_buffer(&output_buffer_desc);

        Self {
            backend: Backend::new(&device, TEXTURE_FORMAT),
            rect_painter: RectPainter::new(),
            device,
            queue,
            frame_desc,
            frame,
            frame_texture_view,
            output_buffer,
            output_buffer_desc,
        }
    }

    pub fn resize(&mut self, size: (u32, u32)) {
        let (width, height) = size;
        self.frame_desc.size.width = width;
        self.frame_desc.size.height = height;

        self.output_buffer_desc.size = (self.get_bytes_per_row() * height) as u64;

        self.frame = self.device.create_texture(&self.frame_desc);
        self.frame_texture_view = self.frame.create_view(&Default::default());
        self.output_buffer = self.device.create_buffer(&self.output_buffer_desc);
    }

    pub fn paint(&mut self) {
        let triangles = &self.rect_painter.vertex_buffers();

        let request = DrawRequest {
            triangles
        };

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("moon wgpu encoder"),
            },
        );

        // Background clear
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("moon::gfx clear bg render pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &self.frame_texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: true
                }
            }],
            depth_stencil_attachment: None
        });
        

        self.backend.draw(
            &self.device,
            &mut encoder,
            &self.queue,
            &self.frame.create_view(&Default::default()),
            (self.frame_desc.size.width, self.frame_desc.size.height),
            request
        );

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.frame,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &self.output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: core::num::NonZeroU32::new(self.get_bytes_per_row()),
                    rows_per_image: core::num::NonZeroU32::new(self.frame_desc.size.height),
                },
            },
            self.frame_desc.size,
        );

        self.queue.submit(Some(encoder.finish()));
    }

    fn get_bytes_per_row(&self) -> u32 {
        let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let unpadded_bytes_per_row = 4 * self.frame_desc.size.width;
        let padding = alignment - unpadded_bytes_per_row % alignment;
        let bytes_per_row = padding + unpadded_bytes_per_row;

        bytes_per_row
    }

    pub async fn output(&mut self) -> Bitmap {
        let buffer_slice = self.output_buffer.slice(..);

        // NOTE: We have to create the mapping THEN device.poll() before await
        // the future. Otherwise the application will freeze.
        let mapping = buffer_slice.map_async(wgpu::MapMode::Read);
        self.device.poll(wgpu::Maintain::Wait);

        mapping.await.unwrap();

        let aligned_output = buffer_slice.get_mapped_range().to_vec();

        let mut output = Vec::new();
        let mut row_pointer: usize = 0;

        let unpadded_bytes_per_row = 4 * self.frame_desc.size.width;

        for _ in 0..self.frame_desc.size.height {
            let row =
                &aligned_output[row_pointer..row_pointer + unpadded_bytes_per_row as usize];
            output.extend_from_slice(row);
            row_pointer += self.get_bytes_per_row() as usize;
        }

        self.output_buffer.unmap();

        output
    }
}

impl<'a> painting::Painter for Painter<'a> {
    fn fill_rect(&mut self, rect: &Rect, color: &Color) {
        self.rect_painter.draw_solid_rect(rect, color);
    }

    fn fill_rrect(&mut self, rect: &RRect, color: &Color) {
        self.rect_painter.draw_solid_rrect(rect, color);
    }
}

