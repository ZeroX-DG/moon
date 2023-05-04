use async_trait::async_trait;
use fontdue::{
    layout::{HorizontalAlign, Layout, LayoutSettings, TextStyle, VerticalAlign},
    Font,
};
use raqote::{DrawOptions, DrawTarget, PathBuilder, SolidSource};
use shared::{
    color::Color,
    primitive::{Point, RRect, Rect, Size},
};

use crate::{
    fonts::{FALLBACK, FALLBACK_BOLD},
    Graphics,
};

pub struct CanvasCPU {
    target: DrawTarget,
    text_layout: Layout,
    default_font: Font,
    default_font_bold: Font,
}

impl CanvasCPU {
    pub fn new() -> Self {
        let mut target = DrawTarget::new(0, 0);
        target.clear(raqote::SolidSource::from_unpremultiplied_argb(
            255, 255, 255, 255,
        ));

        Self {
            target,
            text_layout: Layout::new(fontdue::layout::CoordinateSystem::PositiveYDown),
            default_font: fontdue::Font::from_bytes(FALLBACK, fontdue::FontSettings::default())
                .unwrap(),
            default_font_bold: fontdue::Font::from_bytes(
                FALLBACK_BOLD,
                fontdue::FontSettings::default(),
            )
            .unwrap(),
        }
    }
}

#[async_trait(?Send)]
impl Graphics for CanvasCPU {
    fn fill_rect(&mut self, rect: Rect, color: Color) {
        let src = raqote::Source::Solid(raqote::SolidSource::from_unpremultiplied_argb(
            color.a, color.b, color.g, color.r,
        ));
        let options = DrawOptions::new();
        self.target
            .fill_rect(rect.x, rect.y, rect.width, rect.height, &src, &options);
    }

    fn fill_rrect(&mut self, rect: RRect, color: Color) {
        let corners = &rect.corners;

        let mut path_builder = PathBuilder::new();
        path_builder.move_to(rect.x + rect.corners.top_left.horizontal_r(), rect.y);

        path_builder.line_to(
            rect.x + rect.width - corners.top_right.horizontal_r(),
            rect.y,
        );

        path_builder.quad_to(
            rect.x + rect.width,
            rect.y,
            rect.x + rect.width,
            rect.y + corners.top_right.vertical_r(),
        );

        path_builder.line_to(
            rect.x + rect.width,
            rect.y + rect.height - corners.bottom_right.vertical_r(),
        );

        path_builder.quad_to(
            rect.x + rect.width,
            rect.y + rect.height,
            rect.x + rect.width - corners.bottom_right.horizontal_r(),
            rect.y + rect.height,
        );

        path_builder.line_to(
            rect.x + corners.bottom_left.horizontal_r(),
            rect.y + rect.height,
        );

        path_builder.quad_to(
            rect.x,
            rect.y + rect.height,
            rect.x,
            rect.y + rect.height - corners.bottom_left.vertical_r(),
        );

        path_builder.line_to(rect.x, rect.y + corners.top_left.vertical_r());

        path_builder.quad_to(
            rect.x,
            rect.y,
            rect.x + corners.top_left.horizontal_r(),
            rect.y,
        );

        let path = path_builder.finish();
        let src = raqote::Source::Solid(raqote::SolidSource::from_unpremultiplied_argb(
            color.a, color.b, color.g, color.r,
        ));
        let options = DrawOptions::new();
        self.target.fill(&path, &src, &options);
    }

    fn fill_text(&mut self, content: String, bounds: Rect, color: Color, size: f32, bold: bool) {
        let options = DrawOptions::new();
        self.text_layout.reset(&LayoutSettings {
            x: bounds.x,
            y: bounds.y,
            horizontal_align: HorizontalAlign::Left,
            vertical_align: VerticalAlign::Top,
            ..LayoutSettings::default()
        });
        let font = if bold {
            self.default_font_bold.clone()
        } else {
            self.default_font.clone()
        };
        self.text_layout.append(
            &[font.clone()],
            &TextStyle::new(&content, size * (75. / 96.), 0),
        );

        let mut buf = vec![0; 256 * 256];

        for glyph in self.text_layout.glyphs() {
            let (_, bitmap) = font.rasterize_config(glyph.key);

            let width = glyph.width as i32;
            let height = glyph.height as i32;

            for (i, x) in bitmap.into_iter().enumerate() {
                let src = SolidSource::from_unpremultiplied_argb(
                    (u32::from(x) * u32::from(color.a) / 255) as u8,
                    color.b,
                    color.g,
                    color.r,
                );
                buf[i] = (u32::from(src.a) << 24)
                    | (u32::from(src.r) << 16)
                    | (u32::from(src.g) << 8)
                    | u32::from(src.b);
            }

            let img = raqote::Image {
                width,
                height,
                data: &buf[..],
            };

            self.target.draw_image_with_size_at(
                glyph.width as f32,
                glyph.height as f32,
                glyph.x,
                glyph.y,
                &img,
                &options,
            );
        }
    }

    fn fill_polygon(&mut self, points: Vec<Point>, color: Color) {
        let src = raqote::Source::Solid(raqote::SolidSource::from_unpremultiplied_argb(
            color.a, color.b, color.g, color.r,
        ));
        let options = DrawOptions::new();
        let mut path = raqote::PathBuilder::new();

        let mut point_iter = points.iter();
        let init_point = point_iter.next().unwrap();
        path.move_to(init_point.x, init_point.y);

        for current_point in point_iter {
            path.line_to(current_point.x, current_point.y);
        }

        let path = path.finish();

        self.target.fill(&path, &src, &options);
    }

    fn resize(&mut self, size: Size) {
        self.target = DrawTarget::new(size.width as i32, size.height as i32);
        self.target
            .clear(raqote::SolidSource::from_unpremultiplied_argb(
                255, 255, 255, 255,
            ));
    }

    async fn output(&mut self) -> Vec<u8> {
        self.target.get_data_u8().to_vec()
    }
}
