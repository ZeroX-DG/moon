use crate::request_builder::{PaintBox, PaintText, RectOrRRect, RequestBuilder};
use gfx::Graphics;
use layout::layout_box::LayoutBoxPtr;
use shared::primitive::Size;

pub struct Painter<G: Graphics> {
    gfx: G,
    canvas_size: Size,
}

impl<G: Graphics> Painter<G> {
    pub fn new(gfx: G) -> Self {
        Self {
            gfx,
            canvas_size: Size::default(),
        }
    }

    pub fn resize(&mut self, size: Size) {
        self.gfx.resize(size.clone());
        self.canvas_size = size;
    }

    pub async fn output(&mut self) -> Vec<u8> {
        let result = self.gfx.output().await;
        result
    }

    pub fn paint(&mut self, layout_box: &LayoutBoxPtr) {
        let request = RequestBuilder::new(&self.canvas_size).build(layout_box);

        log::info!("Number of boxes to paint: {}", request.boxes.len());
        log::info!("Number of texts to paint: {}", request.texts.len());

        for current_box in request.boxes {
            self.paint_box(current_box);
        }

        for text in request.texts {
            self.paint_text(text);
        }
    }

    fn paint_text(&mut self, paint_text: PaintText) {
        self.gfx.fill_text(
            paint_text.content,
            paint_text.rect,
            paint_text.color,
            paint_text.font_size,
        );
    }

    fn paint_box(&mut self, paint_box: PaintBox) {
        match paint_box.rect {
            RectOrRRect::Rect(rect) => {
                self.gfx.fill_rect(rect, paint_box.background_color);
            }
            RectOrRRect::RRect(rrect) => {
                self.gfx.fill_rrect(rrect, paint_box.background_color);
            }
        }
    }
}
