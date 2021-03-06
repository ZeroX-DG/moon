mod rect;
mod wgpu_painter;

use painting::{Color, RRect, Rect};
use rect::RectPainter;
use wgpu_painter::WgpuPainter;

pub type OutputBitmap = Vec<u8>;

pub struct Painter {
    backend: WgpuPainter,
    rect_painter: RectPainter,
}

impl Painter {
    pub async fn new(width: u32, height: u32) -> Self {
        let backend = WgpuPainter::new(width, height).await;

        let rect_painter = RectPainter::new(backend.device(), (width, height));

        Self {
            backend,
            rect_painter,
        }
    }

    pub async fn paint(&mut self) -> Option<OutputBitmap> {
        let device = self.backend.device();
        let data = [self.rect_painter.get_paint_data(device)];

        self.backend.paint(&data).await;
        self.backend.output().await
    }
}

impl painting::Painter for Painter {
    fn fill_rect(&mut self, rect: &Rect, color: &Color) {
        self.rect_painter.draw_solid_rect(rect, color);
    }

    fn fill_rrect(&mut self, rect: &RRect, color: &Color) {
        self.rect_painter.draw_solid_rrect(rect, color);
    }

    fn stroke_rect(&mut self, rect: &Rect, color: &Color) {}
}
