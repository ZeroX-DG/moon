use bytemuck::{Pod, Zeroable};
use lyon_tessellation::{FillVertex, FillVertexConstructor, VertexBuffers};
use std::borrow::Cow;
use ultraviolet as uv;

const VERTEX_BUFFER_SIZE: usize = 10_000;
const INDEX_BUFFER_SIZE: usize = 10_000;
const UNIFORM_BUFFER_SIZE: usize = 50;

const INDEX_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint16;
pub type Index = u16;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub pos: uv::Vec2,
    pub color: uv::Vec4,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Uniforms {
    pub screen_size: uv::Vec2,
}

unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}

unsafe impl Pod for Uniforms {}
unsafe impl Zeroable for Uniforms {}

#[derive(Debug)]
struct Buffer<T> {
    label: &'static str,
    raw: wgpu::Buffer,
    size: usize,
    usage: wgpu::BufferUsage,
    _type: std::marker::PhantomData<T>,
}

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: Buffer<Vertex>,
    index_buffer: Buffer<Index>,
    constants: wgpu::BindGroup,
    uniforms_buffer: Buffer<Uniforms>,
}

impl<T> Buffer<T> {
    pub fn new(
        label: &'static str,
        device: &wgpu::Device,
        size: usize,
        usage: wgpu::BufferUsage,
    ) -> Self {
        let raw = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: (std::mem::size_of::<T>() * size) as u64,
            usage,
            mapped_at_creation: false,
        });

        Buffer {
            label,
            raw,
            size,
            usage,
            _type: std::marker::PhantomData,
        }
    }

    pub fn expand(&mut self, device: &wgpu::Device, size: usize) -> bool {
        let needs_resize = self.size < size;

        if needs_resize {
            self.raw = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(self.label),
                size: (std::mem::size_of::<T>() * size) as u64,
                usage: self.usage,
                mapped_at_creation: false,
            });

            self.size = size;
        }

        needs_resize
    }
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("triangle shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/shaders/triangle.wgsl"
            )))),
            flags: wgpu::ShaderFlags::default(),
        });

        let constants_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("moon::gfx::triangle uniforms layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let constants_buffer = Buffer::new(
            "moon::gfx::triangle uniforms buffer",
            device,
            UNIFORM_BUFFER_SIZE,
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let constant_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced_wgpu::triangle uniforms bind group"),
            layout: &constants_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &constants_buffer.raw,
                    offset: 0,
                    size: wgpu::BufferSize::new(std::mem::size_of::<Uniforms>() as u64),
                }),
            }],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("moon::gfx::triangle pipeline layout"),
            bind_group_layouts: &[&constants_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("moon::gfx::triangle pipeline"),
            layout: Some(&layout),

            // Vertex shader
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x2,
                        1 => Float32x4
                    ],
                }],
            },

            // Fragment shader
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        Self {
            pipeline,
            constants: constant_bind_group,
            uniforms_buffer: constants_buffer,
            vertex_buffer: Buffer::new(
                "moon::gfx::triangle vertex buffer",
                device,
                VERTEX_BUFFER_SIZE,
                wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            ),
            index_buffer: Buffer::new(
                "moon::gfx::triangle index buffer",
                device,
                INDEX_BUFFER_SIZE,
                wgpu::BufferUsage::INDEX | wgpu::BufferUsage::COPY_DST,
            ),
        }
    }

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        triangles: &[VertexBuffers<Vertex, Index>],
        target: &wgpu::TextureView,
        size: (u32, u32),
    ) {
        let (total_vertices, total_indices) = triangles
            .iter()
            .map(|buffers| (buffers.vertices.len(), buffers.indices.len()))
            .fold((0, 0), |(total_v, total_i), (v, i)| {
                (total_v + v, total_i + i)
            });

        // Then we ensure the current buffers are big enough, resizing if
        // necessary
        self.vertex_buffer.expand(device, total_vertices);
        self.index_buffer.expand(device, total_indices);

        let mut offsets: Vec<(wgpu::BufferAddress, wgpu::BufferAddress, usize)> =
            Vec::with_capacity(triangles.len());

        let mut last_vertex = 0;
        let mut last_index = 0;

        for buffers in triangles {
            let vertices = bytemuck::cast_slice(&buffers.vertices);
            let indices = bytemuck::cast_slice(&buffers.indices);

            match (
                wgpu::BufferSize::new(vertices.len() as u64),
                wgpu::BufferSize::new(indices.len() as u64),
            ) {
                (Some(_), Some(_)) => {
                    queue.write_buffer(
                        &self.vertex_buffer.raw,
                        (std::mem::size_of::<Vertex>() * last_vertex) as u64,
                        vertices,
                    );
                    queue.write_buffer(
                        &self.index_buffer.raw,
                        (std::mem::size_of::<Index>() * last_index) as u64,
                        indices,
                    );

                    offsets.push((last_vertex as u64, last_index as u64, buffers.indices.len()));

                    last_vertex += buffers.vertices.len();
                    last_index += buffers.indices.len();
                }
                _ => {}
            }
        }

        let uniforms = [Uniforms {
            screen_size: uv::Vec2::new(size.0 as f32, size.1 as f32),
        }];

        let uniforms = bytemuck::cast_slice(&uniforms);

        if wgpu::BufferSize::new(uniforms.len() as u64).is_some() {
            queue.write_buffer(&self.uniforms_buffer.raw, 0, uniforms);
        }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("moon::gfx::triangle renderpass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.constants, &[]);

        for (vertex_offset, index_offset, indices) in offsets {
            let start_index = index_offset * std::mem::size_of::<Index>() as u64;
            let start_vertex = vertex_offset * std::mem::size_of::<Vertex>() as u64;

            render_pass.set_index_buffer(self.index_buffer.raw.slice(start_index..), INDEX_FORMAT);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.raw.slice(start_vertex..));

            render_pass.draw_indexed(0..indices as u32, 0, 0..1);
        }
    }
}

pub struct VertexConstructor;

impl FillVertexConstructor<Vertex> for VertexConstructor {
    fn new_vertex(&mut self, mut vertex: FillVertex) -> Vertex {
        let position = vertex.position().to_array();
        let attrs = vertex.interpolated_attributes();
        Vertex {
            pos: uv::Vec2::from(position),
            color: uv::Vec4::from([
                attrs[0] / 255.0,
                attrs[1] / 255.0,
                attrs[2] / 255.0,
                attrs[3] / 255.0,
            ]),
        }
    }
}
