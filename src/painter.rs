use painting::{Painter, Rect, Paint, PaintStyle, PaintColor};
use skulpin::skia_safe::Canvas;
use skulpin::skia_safe;

pub struct SkiaPainter {
    paint: skia_safe::Paint
}

impl SkiaPainter {
    pub fn new() -> Self {
        let paint = skia_safe::Paint::new(skia_safe::Color4f::new(0., 0., 0., 0.), None);
        Self {
            paint
        }
    }

    pub fn translate_color(color: &PaintColor) -> skia_safe::Color {
        skia_safe::Color::from_argb(
            color.a,
            color.r,
            color.g,
            color.b
        )
    }
}

impl Painter<Canvas> for SkiaPainter {
    fn paint_rect(&mut self, rect: &Rect, paint: &Paint, canvas: &mut Canvas) {
        self.paint.set_style(match paint.style {
            PaintStyle::Fill => skia_safe::PaintStyle::Fill,
            PaintStyle::Stroke => skia_safe::PaintStyle::Stroke
        });

        self.paint.set_color(Self::translate_color(&paint.color));

        canvas.draw_rect(skia_safe::Rect {
            left: rect.x,
            top: rect.y,
            right: rect.x + rect.width,
            bottom: rect.y + rect.height
        }, &self.paint);
    }

    fn clear(&mut self, canvas: &mut Canvas) {
        canvas.clear(skia_safe::Color::from_argb(255, 255, 255, 255));
    }
}
