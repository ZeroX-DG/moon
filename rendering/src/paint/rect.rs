use super::wgpu_painter::{WgpuPaintData, TEXTURE_FORMAT};
use bytemuck::{Pod, Zeroable};
use painting::{Color, Rect, RRect};
use std::borrow::Cow;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

pub struct RectPainter {
    vertices: Vec<Vertex>,
    indexes: Vec<u16>,
    pipeline: wgpu::RenderPipeline,
    uniform_bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    _pos: [f32; 2],
    _color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    _screen_size: [f32; 2],
}

fn vertex(x: f32, y: f32, color: &Color) -> Vertex {
    Vertex {
        _pos: [x, y],
        _color: [
            (color.r as f32) / 255.0,
            (color.g as f32) / 255.0,
            (color.b as f32) / 255.0,
            (color.a as f32) / 255.0,
        ],
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

    let vs_module = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(Cow::Borrowed(
        vs_spirv.as_binary(),
    )));
    let fs_module = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(Cow::Borrowed(
        fs_spirv.as_binary(),
    )));

    (vs_module, fs_module)
}

fn create_pipeline(
    device: &wgpu::Device,
    vs_module: &wgpu::ShaderModule,
    fs_module: &wgpu::ShaderModule,
    uniform_binding_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Rect pipline layout"),
        bind_group_layouts: &[uniform_binding_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Rect render pipeline"),
        layout: Some(&pipeline_layout),
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: vs_module,
            entry_point: "main",
        },
        fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
            module: fs_module,
            entry_point: "main",
        }),
        rasterization_state: None,
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[wgpu::ColorStateDescriptor {
            format: TEXTURE_FORMAT,
            color_blend: wgpu::BlendDescriptor {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha_blend: wgpu::BlendDescriptor {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            write_mask: wgpu::ColorWrite::ALL,
        }],
        depth_stencil_state: None,
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![
                    0 => Float2,
                    1 => Float4
                ],
            }],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });

    pipeline
}

fn create_uniform_bind_group(
    device: &wgpu::Device,
    viewport: (u32, u32),
) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
    let (width, height) = viewport;

    let uniform_binding_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

    let uniforms = Uniforms {
        _screen_size: [width as f32, height as f32],
    };

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform buffer"),
        usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        contents: bytemuck::cast_slice(&[uniforms]),
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &uniform_binding_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(..)),
        }],
    });

    (bind_group, uniform_binding_group_layout)
}

impl RectPainter {
    pub fn new(device: &wgpu::Device, viewport: (u32, u32)) -> Self {
        let (uniform_bind_group, uniform_binding_group_layout) =
            create_uniform_bind_group(device, viewport);
        let (vs_module, fs_module) = create_shaders(device);

        Self {
            vertices: Vec::new(),
            indexes: Vec::new(),
            pipeline: create_pipeline(
                device,
                &vs_module,
                &fs_module,
                &uniform_binding_group_layout,
            ),
            uniform_bind_group,
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
            (rect.x + rect.width, rect.y + rect.height, color),
        ]);

        self.indexes.extend_from_slice(&[
            // first triangle (top_left, top_right, bottom_left)
            indexes[0], indexes[1], indexes[2],
            // second triangle (top_right, bottom_right, bottom_left)
            indexes[1], indexes[3], indexes[2],
        ]);
    }

    pub fn draw_solid_rrect(&mut self, rect: &RRect, color: &Color) {
        // We gonna use 9-slice scaling
        // See: https://en.wikipedia.org/wiki/9-slice_scaling
        // 
        // So our rrect will be structured into 9 parts:
        // 1 | 2 | 3
        // 4 | 5 | 6
        // 7 | 8 | 9
        //
        // With 1 3 7 9 is round corner and the rest are rects

        // rects
        // part 2
        // top left = (rect.x + rect.radius.top_left, rect.y)
        // top right = (rect.x + rect.width - rect.radius.top_right, rect.y)
        // bottom left = (rect.x + rect.radius.top_left, rect.y + rect.radius.top_left)
        // bottom right = (rect.x + rect.width - rect.radius.top_right, rect.y + rect.radius.top_right)
        let rect_2_indexes = self.create_vertices(&[
            // top_left
            (rect.x + rect.corners.top_left.horizontal_r(), rect.y, color),
            // top_right
            (rect.x + (rect.width - rect.corners.top_right.horizontal_r()), rect.y, color),
            // bottom_left
            (rect.x + rect.corners.top_left.horizontal_r(), rect.y + rect.corners.top_left.vertical_r(), color),
            // bottom_right
            (rect.x + (rect.width - rect.corners.top_right.horizontal_r()), rect.y + rect.corners.top_right.vertical_r(), color),
        ]);

        self.indexes.extend_from_slice(&[
            // first triangle (top_left, top_right, bottom_left)
            rect_2_indexes[0], rect_2_indexes[1], rect_2_indexes[2],
            // second triangle (top_right, bottom_right, bottom_left)
            rect_2_indexes[1], rect_2_indexes[3], rect_2_indexes[2],
        ]);

        // part 8
        // top left = (rect.x + rect.radius.bottom_left, rect.y - rect.radius.bottom_left)
        // top right = (rect.x + rect.width - rect.radius.bottom_right, rect.y - rect.radius.bottom_right)
        // bottom left = (rect.x + rect.radius.bottom_left, rect.y + rect.height)
        // bottom right = (rect.x + rect.width - rect.radius.bottom_right, rect.y + rect.height)
        let rect_8_indexes = self.create_vertices(&[
            // top_left
            (rect.x + rect.corners.bottom_left.horizontal_r(), rect.y + (rect.height - rect.corners.bottom_left.vertical_r()), color),
            // top_right
            (rect.x + (rect.width - rect.corners.bottom_right.horizontal_r()), rect.y + (rect.height - rect.corners.bottom_right.vertical_r()), color),
            // bottom_left
            (rect.x + rect.corners.bottom_left.horizontal_r(), rect.y + rect.height, color),
            // bottom_right
            (rect.x + (rect.width - rect.corners.bottom_right.horizontal_r()), rect.y + rect.height, color),
        ]);

        self.indexes.extend_from_slice(&[
            // first triangle (top_left, top_right, bottom_left)
            rect_8_indexes[0], rect_8_indexes[1], rect_8_indexes[2],
            // second triangle (top_right, bottom_right, bottom_left)
            rect_8_indexes[1], rect_8_indexes[3], rect_8_indexes[2],
        ]);

        // part 4
        // top left = (rect.x, rect.y + rect.radius.top_left)
        // top right = (rect.x + rect.radius.top_left, rect.y + rect.radius.top_left)
        // bottom left = (rect.x, rect.y + rect.height - rect.radius.bottom_left)
        // bottom right = (rect.x + rect.radius.bottom_left, rect.y + rect.height - rect.radius.bottom_left)
        let rect_4_indexes = self.create_vertices(&[
            // top_left
            (rect.x, rect.y + rect.corners.top_left.vertical_r(), color),
            // top_right
            (rect.x + rect.corners.top_left.horizontal_r(), rect.y + rect.corners.top_left.vertical_r(), color),
            // bottom_left
            (rect.x, rect.y + rect.height - rect.corners.bottom_left.vertical_r(), color),
            // bottom_right
            (rect.x + rect.corners.bottom_left.horizontal_r(), rect.y + rect.height - rect.corners.bottom_left.vertical_r(), color),
        ]);

        self.indexes.extend_from_slice(&[
            // first triangle (top_left, top_right, bottom_left)
            rect_4_indexes[0], rect_4_indexes[1], rect_4_indexes[2],
            // second triangle (top_right, bottom_right, bottom_left)
            rect_4_indexes[1], rect_4_indexes[3], rect_4_indexes[2],
        ]);

        // part 6
        // top left = (rect.x + rect.width - rect.radius_top_right, rect.y + rect.radius.top_right)
        // top right = (rect.x + rect.width, rect.y + rect.radius.top_right)
        // bottom left = (rect.x + rect.width - rect.radius_bottom_right, rect.y + rect.height - rect.radius.bottom_right)
        // bottom right = (rect.x + rect.width, rect.y + rect.height - rect.radius.bottom_right)
        let rect_6_indexes = self.create_vertices(&[
            // top_left
            (rect.x + rect.width - rect.corners.top_right.horizontal_r(), rect.y + rect.corners.top_right.vertical_r(), color),
            // top_right
            (rect.x + rect.width, rect.y + rect.corners.top_right.vertical_r(), color),
            // bottom_left
            (rect.x + rect.width - rect.corners.bottom_right.horizontal_r(), rect.y + rect.height - rect.corners.bottom_right.vertical_r(), color),
            // bottom_right
            (rect.x + rect.width, rect.y + rect.height - rect.corners.bottom_right.vertical_r(), color),
        ]);

        self.indexes.extend_from_slice(&[
            // first triangle (top_left, top_right, bottom_left)
            rect_6_indexes[0], rect_6_indexes[1], rect_6_indexes[2],
            // second triangle (top_right, bottom_right, bottom_left)
            rect_6_indexes[1], rect_6_indexes[3], rect_6_indexes[2],
        ]);

        // part 5
        // top left = (rect.x + rect.radius.top_left, rect.y + rect.radius.top_left)
        // top right = (rect.x + rect.width - rect.radius.top_right, rect.y + rect.radius.top_right)
        // bottom left = (rect.x + rect.radius_bottom_left, rect.y + rect.height - rect.radius.bottom_left)
        // bottom right = (rect.x + rect.width - rect.radius_bottom_right, rect.y + rect.height - rect.radius.bottom_right)
        let rect_5_indexes = self.create_vertices(&[
            // top_left
            (rect.x + rect.corners.top_left.horizontal_r(), rect.y + rect.corners.top_left.vertical_r(), color),
            // top_right
            (rect.x + rect.width - rect.corners.top_right.horizontal_r(), rect.y + rect.corners.top_right.vertical_r(), color),
            // bottom_left
            (rect.x + rect.corners.bottom_left.horizontal_r(), rect.y + rect.height - rect.corners.bottom_left.vertical_r(), color),
            // bottom_right
            (rect.x + rect.width - rect.corners.bottom_right.horizontal_r(), rect.y + rect.height - rect.corners.bottom_right.vertical_r(), color),
        ]);

        self.indexes.extend_from_slice(&[
            // first triangle (top_left, top_right, bottom_left)
            rect_5_indexes[0], rect_5_indexes[1], rect_5_indexes[2],
            // second triangle (top_right, bottom_right, bottom_left)
            rect_5_indexes[1], rect_5_indexes[3], rect_5_indexes[2],
        ]);

        // corner
        // part 1
        let mut points = Vec::new();
        for angle in (0..=90).step_by(1) {
            let radian = angle as f32 * std::f32::consts::PI / 180.0;
            let x = -radian.cos() * rect.corners.top_left.horizontal_r();
            let y = -radian.sin() * rect.corners.top_left.vertical_r();

            points.push((rect.x + rect.corners.top_left.horizontal_r() + x, rect.y + rect.corners.top_left.vertical_r() + y, color));
        }
        let indexes = self.create_vertices(&points);
        for i in 0..indexes.len() - 1 {
            self.indexes.extend_from_slice(&[
                indexes[i], indexes[i + 1], rect_5_indexes[0]
            ]);
        }

        // part 3
        let mut points = Vec::new();
        for angle in (0..=90).step_by(1) {
            let radian = angle as f32 * std::f32::consts::PI / 180.0;
            let x = radian.cos() * rect.corners.top_right.horizontal_r();
            let y = -radian.sin() * rect.corners.top_right.vertical_r();

            points.push((rect.x + rect.width - rect.corners.top_right.horizontal_r() + x, rect.y + rect.corners.top_right.vertical_r() + y, color));
        }
        let indexes = self.create_vertices(&points);
        for i in 0..indexes.len() - 1 {
            self.indexes.extend_from_slice(&[
                indexes[i], indexes[i + 1], rect_5_indexes[1]
            ]);
        }

        // part 7
        let mut points = Vec::new();
        for angle in (0..=90).step_by(1) {
            let radian = angle as f32 * std::f32::consts::PI / 180.0;
            let x = -radian.cos() * rect.corners.bottom_left.horizontal_r();
            let y = radian.sin() * rect.corners.bottom_left.vertical_r();

            points.push((rect.x + rect.corners.bottom_left.horizontal_r() + x, rect.y + rect.height - rect.corners.bottom_left.vertical_r() + y, color));
        }
        let indexes = self.create_vertices(&points);
        for i in 0..indexes.len() - 1 {
            self.indexes.extend_from_slice(&[
                indexes[i], indexes[i + 1], rect_5_indexes[2]
            ]);
        }

        // part 9
        let mut points = Vec::new();
        for angle in (0..=90).step_by(1) {
            let radian = angle as f32 * std::f32::consts::PI / 180.0;
            let x = radian.cos() * rect.corners.bottom_right.horizontal_r();
            let y = radian.sin() * rect.corners.bottom_right.vertical_r();

            points.push((rect.x + rect.width - rect.corners.bottom_right.horizontal_r() + x, rect.y + rect.height - rect.corners.bottom_right.vertical_r() + y, color));
        }
        let indexes = self.create_vertices(&points);
        for i in 0..indexes.len() - 1 {
            self.indexes.extend_from_slice(&[
                indexes[i], indexes[i + 1], rect_5_indexes[3]
            ]);
        }
    }

    pub fn get_paint_data(&self, device: &wgpu::Device) -> WgpuPaintData {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Rect vertext buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Rect index buffer"),
            contents: bytemuck::cast_slice(&self.indexes),
            usage: wgpu::BufferUsage::INDEX,
        });

        WgpuPaintData {
            vertex_buffer,
            index_buffer,
            pipeline: &self.pipeline,
            bind_group: &self.uniform_bind_group,
            nums_indexes: self.indexes.len() as u32,
        }
    }
}
