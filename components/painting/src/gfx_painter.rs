use async_trait::async_trait;
use shared::color::Color;
use shared::primitive::*;

#[async_trait(?Send)]
pub trait GfxPainter {
    fn fill_rect(&mut self, rect: Rect, color: Color);
    fn fill_rrect(&mut self, rect: RRect, color: Color);
    fn fill_text(&mut self, content: String, bounds: Rect, color: Color, size: f32);
    fn resize(&mut self, size: Size);
    async fn output(&mut self) -> Vec<u8>;
}
