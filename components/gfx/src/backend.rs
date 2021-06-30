use super::triangle;
use lyon_tessellation::VertexBuffers;

pub struct Backend {
    triangle_pipeline: triangle::Pipeline,
}

pub struct DrawRequest<'a> {
    pub triangles: &'a [VertexBuffers<triangle::Vertex, triangle::Index>],
}

impl Backend {
    pub fn new(device: &wgpu::Device, texture_format: wgpu::TextureFormat) -> Self {
        Self {
            triangle_pipeline: triangle::Pipeline::new(device, texture_format)
        }
    }

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue, 
        target: &wgpu::TextureView, 
        size: (u32, u32),
        request: DrawRequest
    ) {
        if !request.triangles.is_empty() {
            self.triangle_pipeline.draw(
                device,
                encoder,
                queue,
                &request.triangles,
                target,
                size
            );
        }
    }
}

