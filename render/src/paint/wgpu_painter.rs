use super::OutputBitmap;

pub struct WgpuPainter {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

pub struct WgpuPaintData {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub nums_indexes: u32,
}

pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

impl WgpuPainter {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: None,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&Default::default(), None)
            .await
            .unwrap();

        Self {
            device,
            queue,
        }
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub async fn paint(&mut self, size: (u32, u32), data: &[WgpuPaintData]) -> wgpu::Buffer {
        let (width, height) = size;
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,
                height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TEXTURE_FORMAT,
            usage: wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });
        let texture_view = texture.create_view(&Default::default());

        let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let unpadded_bytes_per_row = 4 * width;
        let padding = alignment - unpadded_bytes_per_row % alignment;
        let bytes_per_row = padding + unpadded_bytes_per_row;

        let output_buffer_size = (bytes_per_row * height) as wgpu::BufferAddress;

        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::MAP_READ,
            label: None,
            mapped_at_creation: false,
        };
        let output_buffer = self.device.create_buffer(&output_buffer_desc);
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Main encoder"),
            });

        {
            let render_pass_desc = wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            };
            let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

            for paint_data in data {
                render_pass.set_bind_group(0, &paint_data.bind_group, &[]);
                render_pass.set_pipeline(&paint_data.pipeline);
                render_pass.set_index_buffer(paint_data.index_buffer.slice(..));
                render_pass.set_vertex_buffer(0, paint_data.vertex_buffer.slice(..));
                render_pass.draw_indexed(0..paint_data.nums_indexes, 0, 0..1);
            }
        }

        encoder.copy_texture_to_buffer(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::BufferCopyView {
                buffer: &output_buffer,
                layout: wgpu::TextureDataLayout {
                    offset: 0,
                    bytes_per_row,
                    rows_per_image: height,
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        output_buffer
    }

    pub async fn output(&mut self, size: (u32, u32), output_buffer: wgpu::Buffer) -> Option<OutputBitmap> {
        // NOTE: We have to create the mapping THEN device.poll(). If we don't
        // the application will freeze.
        let mapping = output_buffer.slice(..).map_async(wgpu::MapMode::Read);
        self.device.poll(wgpu::Maintain::Wait);

        match mapping.await {
            Ok(()) => {
                // Because the output data has aligned with wgpu::COPY_BYTES_PER_ROW_ALIGNMENT,
                // we need to "unalign" the output data so it become valid data.
                //
                // TODO: Remove this step when we don't have to align the data anymore.
                // See: https://github.com/gfx-rs/wgpu/issues/988
                let aligned_output = output_buffer.slice(..).get_mapped_range().to_vec();

                let (width, height) = size;
                let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
                let unpadded_bytes_per_row = 4 * width;
                let padding = alignment - unpadded_bytes_per_row % alignment;
                let bytes_per_row = padding + unpadded_bytes_per_row;

                let mut output = Vec::new();

                let mut row_pointer: usize = 0;

                for _ in 0..height {
                    let row =
                        &aligned_output[row_pointer..row_pointer + unpadded_bytes_per_row as usize];
                    output.extend_from_slice(row);
                    row_pointer += bytes_per_row as usize;
                }

                output_buffer.unmap();

                Some(output)
            }
            Err(e) => {
                log::error!("Error while getting output: {}", e);
                None
            }
        }
    }
}
