use super::triangle;
use super::text;
use lyon_tessellation::VertexBuffers;

pub struct Backend {
    triangle_pipeline: triangle::Pipeline,
    text_pipeline: text::Pipeline,
}

pub struct DrawRequest<'a> {
    pub triangles: &'a [VertexBuffers<triangle::Vertex, triangle::Index>],
    pub texts: &'a [text::Text]
}

impl Backend {
    pub fn new(device: &wgpu::Device, texture_format: wgpu::TextureFormat) -> Self {
        Self {
            triangle_pipeline: triangle::Pipeline::new(device, texture_format),
            text_pipeline: text::Pipeline::new(device, texture_format, None)
        }
    }

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        staging_belt: &mut wgpu::util::StagingBelt,
        target: &wgpu::TextureView,
        size: (u32, u32),
        request: DrawRequest,
    ) {
        if !request.triangles.is_empty() {
            self.triangle_pipeline.draw(
                device,
                encoder,
                staging_belt,
                &request.triangles,
                target,
                size,
            );
        }

        if !request.texts.is_empty() {
            for text in request.texts {
                let section = wgpu_glyph::Section {
                    screen_position: (text.bounds.x, text.bounds.y),
                    bounds: (text.bounds.width, text.bounds.height),
                    text: vec![wgpu_glyph::Text {
                        text: &text.content,
                        scale: wgpu_glyph::ab_glyph::PxScale {
                            x: text.size,
                            y: text.size,
                        },
                        font_id: wgpu_glyph::FontId(0),
                        extra: wgpu_glyph::Extra {
                            color: text.color.clone().into(),
                            z: 0.0,
                        },
                    }],
                    layout: wgpu_glyph::Layout::default()
                        .h_align(wgpu_glyph::HorizontalAlign::Left)
                        .v_align(wgpu_glyph::VerticalAlign::Top),
                    ..Default::default()
                };
                self.text_pipeline.queue(section)
            }
            self.text_pipeline.draw_queued(device, staging_belt, encoder, target, size);
        }
    }
}
