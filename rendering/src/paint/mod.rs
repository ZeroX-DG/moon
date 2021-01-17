mod wgpu_painter;
mod rect;

use wgpu_painter::WgpuPainter;
use painting::{Rect, Color};
use rect::RectPainter;

pub type OutputBitmap = Vec<u8>;

pub struct Painter {
    backend: WgpuPainter,
    rect_painter: RectPainter,
}

impl Painter {
    pub async fn new(width: u32, height: u32) -> Self {
        let backend = WgpuPainter::new(width, height).await;

        let rect_painter = RectPainter::new(backend.device());

        Self {
            backend,
            rect_painter 
        }
    }

    pub async fn paint(&mut self) -> Option<OutputBitmap> {
        let device = self.backend.device();
        let data = [
            self.rect_painter.get_paint_data(device)
        ];

        let unpadded_bytes_per_row = 4 * 500;
        let padding = 256 - unpadded_bytes_per_row % 256;
        let bytes_per_row = padding + unpadded_bytes_per_row;

        self.backend.paint(&data).await;
        match self.backend.output().await {
            Some(data) => {
                let mut result = Vec::new();
                let mut pointer = 0;
                for _ in 0..300 {
                    let row = &data[pointer..pointer + unpadded_bytes_per_row];
                    result.extend_from_slice(row);
                    pointer += bytes_per_row;
                }

                Some(result)
            },
            None => None
        }
    }
}



impl painting::Painter for Painter {
    fn fill_rect(&mut self, rect: &Rect, color: &Color) {
        self.rect_painter.draw_solid_rect(rect, color);
    }

    fn stroke_rect(&mut self, rect: &Rect, color: &Color) {
        
    }
}