use painting::{Painter, Rect, Paint, PaintStyle};
use skulpin::skia_safe::Canvas;
use skulpin::skia_safe;

pub struct SkiaPainter<'a> {
    canvas: &'a mut Canvas,
    paint: skia_safe::Paint
}

impl<'a> SkiaPainter<'a> {
    pub fn new(canvas: &'a mut Canvas) -> Self {
        let paint = skia_safe::Paint::new(skia_safe::Color4f::new(0., 0., 0., 0.), None);
        Self {
            canvas,
            paint
        }
    }
}

impl<'a> Painter for SkiaPainter<'a> {
    fn paint_rect(&mut self, rect: Rect, paint: Paint) {
        self.paint.set_style(match paint.style {
            PaintStyle::Fill => skia_safe::PaintStyle::Fill,
            PaintStyle::Stroke => skia_safe::PaintStyle::Stroke
        });

        self.paint.set_color(skia_safe::Color::from_argb(paint.color.a, paint.color.r, paint.color.g, paint.color.b));

        self.canvas.draw_rect(skia_safe::Rect {
            left: rect.x,
            top: rect.y,
            right: rect.x + rect.width,
            bottom: rect.y + rect.height
        }, &self.paint);
    }

    fn clear(&mut self) {
        self.canvas.clear(skia_safe::Color::from_argb(255, 255, 255, 255));
    }
}

