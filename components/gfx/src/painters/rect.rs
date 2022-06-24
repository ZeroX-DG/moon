use lyon_tessellation::geom::point;
use lyon_tessellation::path::Path;
use shared::color::Color;
use shared::primitive::{RRect, Rect};

use crate::tessellator::Tessellator;

pub struct RectPainter;

impl RectPainter {
    pub fn new() -> Self {
        Self
    }

    pub fn draw_solid_rect(&mut self, tessellator: &mut Tessellator, rect: &Rect, color: &Color) {
        let color_arr: [f32; 4] = [
            color.r.into(),
            color.g.into(),
            color.b.into(),
            color.a.into(),
        ];

        let mut path_builder = Path::builder_with_attributes(4);
        path_builder.begin(point(rect.x, rect.y), &color_arr);
        path_builder.line_to(point(rect.x + rect.width, rect.y), &color_arr);
        path_builder.line_to(point(rect.x + rect.width, rect.y + rect.height), &color_arr);
        path_builder.line_to(point(rect.x, rect.y + rect.height), &color_arr);
        path_builder.end(true);

        let path = path_builder.build();
        tessellator.tessellate_path(path);
    }

    pub fn draw_solid_rrect(&mut self, tessellator: &mut Tessellator, rect: &RRect, color: &Color) {
        let color_arr: [f32; 4] = [
            color.r.into(),
            color.g.into(),
            color.b.into(),
            color.a.into(),
        ];

        let corners = &rect.corners;

        let mut path_builder = Path::builder_with_attributes(4);
        path_builder.begin(
            point(rect.x + rect.corners.top_left.horizontal_r(), rect.y),
            &color_arr,
        );

        path_builder.line_to(
            point(
                rect.x + rect.width - corners.top_right.horizontal_r(),
                rect.y,
            ),
            &color_arr,
        );

        path_builder.quadratic_bezier_to(
            point(rect.x + rect.width, rect.y),
            point(rect.x + rect.width, rect.y + corners.top_right.vertical_r()),
            &color_arr,
        );

        path_builder.line_to(
            point(
                rect.x + rect.width,
                rect.y + rect.height - corners.bottom_right.vertical_r(),
            ),
            &color_arr,
        );

        path_builder.quadratic_bezier_to(
            point(rect.x + rect.width, rect.y + rect.height),
            point(
                rect.x + rect.width - corners.bottom_right.horizontal_r(),
                rect.y + rect.height,
            ),
            &color_arr,
        );

        path_builder.line_to(
            point(
                rect.x + corners.bottom_left.horizontal_r(),
                rect.y + rect.height,
            ),
            &color_arr,
        );

        path_builder.quadratic_bezier_to(
            point(rect.x, rect.y + rect.height),
            point(
                rect.x,
                rect.y + rect.height - corners.bottom_left.vertical_r(),
            ),
            &color_arr,
        );

        path_builder.line_to(
            point(rect.x, rect.y + corners.top_left.vertical_r()),
            &color_arr,
        );

        path_builder.quadratic_bezier_to(
            point(rect.x, rect.y),
            point(rect.x + corners.top_left.horizontal_r(), rect.y),
            &color_arr,
        );

        path_builder.end(true);

        let path = path_builder.build();
        tessellator.tessellate_path(path);
    }
}

