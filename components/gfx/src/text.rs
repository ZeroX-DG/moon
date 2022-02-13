use shared::{color::Color, primitive::rect::Rect};
use wgpu_glyph::ab_glyph;
use crate::fonts::FALLBACK;

pub struct Text {
    pub content: String,
    pub bounds: Rect,
    pub size: f32,
    pub color: Color,
}

pub struct Pipeline {
    draw_brush: wgpu_glyph::GlyphBrush<()>,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        default_font: Option<&[u8]>,
    ) -> Self {
        let default_font = default_font.map(|slice| slice.to_vec());
        let default_font = default_font.unwrap_or_else(|| FALLBACK.to_vec());

        let font = ab_glyph::FontArc::try_from_vec(default_font).unwrap_or_else(|_| {
            log::warn!(
                "System font failed to load. Falling back to \
                    embedded font..."
            );

            ab_glyph::FontArc::try_from_slice(FALLBACK).expect("Load fallback font")
        });

        let draw_brush = wgpu_glyph::GlyphBrushBuilder::using_font(font)
            .initial_cache_size((2048, 2048))
            .draw_cache_multithread(true)
            .build(device, format);

        Self { draw_brush }
    }

    pub fn queue(&mut self, section: wgpu_glyph::Section<'_>) {
        self.draw_brush.queue(section);
    }

    pub fn draw_queued(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        size: (u32, u32),
    ) {
        let (width, height) = size;
        self.draw_brush
            .draw_queued(device, staging_belt, encoder, target, width, height)
            .expect("Draw text");
    }
}
