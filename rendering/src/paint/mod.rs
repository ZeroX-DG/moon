mod rect;

use painting::{Rect, Color};
use rect::RectPainter;

pub struct Painter {
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: (u32, u32),
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    rect_painter: RectPainter,
    output_buffer: wgpu::Buffer
}

impl Painter {
    pub async fn new(width: u32, height: u32) -> Self {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: None
            }
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &Default::default(),
            None
        ).await.unwrap();

        let texture_descriptor = wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,
                height,
                depth: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::OUTPUT_ATTACHMENT
        };

        let texture = device.create_texture(&texture_descriptor);
        let texture_view = texture.create_view(&Default::default());

        let rect_painter = RectPainter::new(&device);

        let u32_size = std::mem::size_of::<u32>() as u32;

        let output_buffer_size = (u32_size * width * height) as wgpu::BufferAddress;
        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsage::COPY_DST
                // this tells wpgu that we want to read this buffer from the cpu
                | wgpu::BufferUsage::MAP_READ,
            label: None,
            mapped_at_creation: false
        };
        let output_buffer = device.create_buffer(&output_buffer_desc);

        Self {
            device,
            queue,
            size: (width, height),
            texture,
            texture_view,
            rect_painter,
            output_buffer,
        }
    }

    pub async fn paint(&mut self) -> Vec<u8> {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let rect_buffer = self.rect_painter.buffer(&self.device).await;

        {
            let render_pass_desc = wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &self.texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: true
                    }
                }],
                depth_stencil_attachment: None,
            };
            let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

            render_pass.set_vertex_buffer(0, rect_buffer.slice(..));
            render_pass.set_pipeline(&self.rect_painter.pipeline());
            //render_pass.draw(0.., 0..1);
        }

        let u32_size = std::mem::size_of::<u32>() as u32;

        encoder.copy_texture_to_buffer(
            wgpu::TextureCopyView {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::BufferCopyView {
                buffer: &self.output_buffer,
                layout: wgpu::TextureDataLayout {
                    offset: 0,
                    bytes_per_row: u32_size * self.size.0,
                    rows_per_image: self.size.1,
                }
            },
            wgpu::Extent3d {
                width: self.size.0,
                height: self.size.1,
                depth: 1
            },
        );

        self.queue.submit(vec![encoder.finish()]);

        let mapping = self.output_buffer.slice(..).map_async(wgpu::MapMode::Read);
        self.device.poll(wgpu::Maintain::Wait);

        mapping.await.unwrap();
        
        self.output_buffer.slice(..).get_mapped_range().to_vec()
    }
}

impl painting::Painter for Painter {
    fn clear(&mut self) {
        
    }

    fn fill_rect(&mut self, rect: &Rect, color: &Color) {
        self.rect_painter.handle_fill_rect(rect, color);
    }

    fn stroke_rect(&mut self, rect: &Rect, color: &Color) {
        
    }
}