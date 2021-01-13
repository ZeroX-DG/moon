use std::borrow::Cow;

use painting::{Color, Rect};

pub struct RectPainter {
    pipeline: wgpu::RenderPipeline,
    vertices: Vec<f32>
}

impl RectPainter {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            pipeline: create_rendering_pipeline(device),
            vertices: Vec::new()
        }
    }

    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    pub async fn buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: self.vertices.len() as u64 * 2,
            usage: wgpu::BufferUsage::VERTEX,
            mapped_at_creation: false
        });

        let map_wait  = buffer.slice(..).map_async(wgpu::MapMode::Write);

        device.poll(wgpu::Maintain::Wait);

        map_wait.await.unwrap();

        buffer.slice(..).get_mapped_range_mut().copy_from_slice(to_byte_slice(self.vertices.as_slice()));

        buffer
    }

    pub fn handle_fill_rect(&mut self, rect: &Rect, color: &Color) {
        // top left
        self.vertices.extend_from_slice(&[
            rect.x,
            rect.y
        ]);

        // top right
        self.vertices.extend_from_slice(&[
            rect.x + rect.width,
            rect.y
        ]);

        // bottom left
        self.vertices.extend_from_slice(&[
            rect.x,
            rect.y + rect.height
        ]);

        // -----------------
        // top right
        self.vertices.extend_from_slice(&[
            rect.x + rect.width,
            rect.y
        ]);

        // bottom left
        self.vertices.extend_from_slice(&[
            rect.x,
            rect.y + rect.height
        ]);

        // bottom right
        self.vertices.extend_from_slice(&[
            rect.x + rect.width,
            rect.y + rect.height
        ]);
    }
}

fn to_byte_slice<'a>(floats: &'a [f32]) -> &'a [u8] {
    unsafe {
        std::slice::from_raw_parts(floats.as_ptr() as *const _, floats.len() * 4)
    }
}

fn create_rendering_pipeline(device: &wgpu::Device) -> wgpu::RenderPipeline {
    let vs_src = include_str!("../../shaders/triangle.vert");
    let fs_src = include_str!("../../shaders/triangle.frag");
    let mut compiler = shaderc::Compiler::new().unwrap();
    let vs_spirv = compiler
        .compile_into_spirv(
            vs_src,
            shaderc::ShaderKind::Vertex,
            "shader.vert",
            "main",
            None,
        )
        .unwrap();
    let fs_spirv = compiler
        .compile_into_spirv(
            fs_src,
            shaderc::ShaderKind::Fragment,
            "shader.frag",
            "main",
            None,
        )
        .unwrap();
    let vs_data = wgpu::ShaderModuleSource::SpirV(Cow::Borrowed(vs_spirv.as_binary()));
    let fs_data = wgpu::ShaderModuleSource::SpirV(Cow::Borrowed(fs_spirv.as_binary()));
    let vs_module = device.create_shader_module(vs_data);
    let fs_module = device.create_shader_module(fs_data);

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        push_constant_ranges: &[],
        bind_group_layouts: &[],
    });

    return device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&render_pipeline_layout),
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: &vs_module,
            entry_point: "main",
        },
        fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
            module: &fs_module,
            entry_point: "main",
        }),
        rasterization_state: None,
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[wgpu::ColorStateDescriptor {
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }],
        depth_stencil_state: None,
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });
}