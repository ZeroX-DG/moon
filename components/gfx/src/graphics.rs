use async_trait::async_trait;
use shared::color::Color;
use shared::primitive::*;

#[async_trait(?Send)]
pub trait Graphics {
    fn fill_rect(&mut self, rect: Rect, color: Color);
    fn fill_rrect(&mut self, rect: RRect, color: Color);
    fn fill_text(&mut self, content: String, bounds: Rect, color: Color, size: f32, bold: bool);
    fn fill_polygon(&mut self, points: Vec<Point>, color: Color);
    fn resize(&mut self, size: Size);
    async fn output(&mut self) -> Vec<u8>;
}
