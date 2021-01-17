use std::borrow::Cow;
use painting::{Color, Rect};
use bytemuck::{Pod, Zeroable};
use super::wgpu_painter::{TEXTURE_FORMAT, WgpuPaintData};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

pub struct RectPainter {
    vertices: Vec<Vertex>,
    indexes: Vec<u16>,
    pipeline: wgpu::RenderPipeline
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    _pos: [f32; 2],
    _color: [f32; 4]
}

fn vertex(x: f32, y: f32, color: &Color) -> Vertex {
    Vertex {
        _pos: [x, y],
        _color: [
            (color.r as f32) / 255.0,
            (color.g as f32) / 255.0,
            (color.b as f32) / 255.0,
            (color.a as f32) / 255.0
        ]
    }
}

fn create_shaders(device: &wgpu::Device) -> (wgpu::ShaderModule, wgpu::ShaderModule) {
    let vs_src = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/triangle.vert"
    ));

    let fs_src = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/triangle.frag"
    ));

    let mut compiler = shaderc::Compiler::new().unwrap();

    let vs_spirv = compiler
        .compile_into_spirv(
            vs_src,
            shaderc::ShaderKind::Vertex,
            "triangle.vert",
            "main",
            None,
        )
        .unwrap();
    let fs_spirv = compiler
        .compile_into_spirv(
            fs_src,
            shaderc::ShaderKind::Fragment,
            "triangle.frag",
            "main",
            None,
        )
        .unwrap();

    let vs_module = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(Cow::Borrowed(vs_spirv.as_binary())));
    let fs_module = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(Cow::Borrowed(fs_spirv.as_binary())));

    (vs_module, fs_module)
}

fn create_pipeline(device: &wgpu::Device, vs_module: &wgpu::ShaderModule, fs_module: &wgpu::ShaderModule) -> wgpu::RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Rect pipline layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[]
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Rect render pipeline"),
        layout: Some(&pipeline_layout),
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: vs_module,
            entry_point: "main"
        },
        fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
            module: fs_module,
            entry_point: "main",
        }),
        rasterization_state: None,
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[wgpu::ColorStateDescriptor {
            format: TEXTURE_FORMAT,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }],
        depth_stencil_state: None,
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[
                wgpu::VertexBufferDescriptor {
                    stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float2,
                        1 => Float4
                    ],
                }
            ],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });

    pipeline
}

impl RectPainter {
    pub fn new(device: &wgpu::Device) -> Self {
        let (vs_module, fs_module) = create_shaders(device);

        Self {
            vertices: Vec::new(),
            indexes: Vec::new(),
            pipeline: create_pipeline(device, &vs_module, &fs_module)
        }
    }

    fn create_vertices(&mut self, vertices: &[(f32, f32, &Color)]) -> Vec<u16> {
        let mut indexes = Vec::new();
        for (x, y, color) in vertices {
            indexes.push(self.vertices.len() as u16);
            self.vertices.push(vertex(*x, *y, color));
        }
        indexes
    }

    pub fn draw_solid_rect(&mut self, rect: &Rect, color: &Color) {
        let indexes = self.create_vertices(&[
            // top_left
            (rect.x, rect.y, color),

            // top_right
            (rect.x + rect.width, rect.y, color),

            // bottom_left
            (rect.x, rect.y + rect.height, color),

            // bottom_right
            (rect.x + rect.width, rect.y + rect.height, color)
        ]);

        self.indexes.extend_from_slice(&[
            // first triangle (top_left, top_right, bottom_left)
            indexes[0], indexes[1], indexes[2],
            // second triangle (top_right, bottom_right, bottom_left)
            indexes[1], indexes[3], indexes[2]
        ]);
    }

    pub fn get_paint_data(&self, device: &wgpu::Device) -> WgpuPaintData {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Rect vertext buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsage::VERTEX
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Rect index buffer"),
            contents: bytemuck::cast_slice(&self.indexes),
            usage: wgpu::BufferUsage::INDEX
        });

        WgpuPaintData {
            vertex_buffer,
            index_buffer,
            pipeline: &self.pipeline,
            nums_vertices: self.vertices.len() as u32
        }
    }
}