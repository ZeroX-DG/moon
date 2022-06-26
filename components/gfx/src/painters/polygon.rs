use lyon_tessellation::{path::Path, geom::point};
use shared::{primitive::Point, color::Color};

use crate::tessellator::Tessellator;

pub struct PolygonPainter;

impl PolygonPainter {
    pub fn new() -> Self {
        Self
    }

    pub fn fill_polygon(
        &self,
        tessellator: &mut Tessellator,
        points: &[Point],
        color: &Color
    ) {
        if points.len() < 3 {
            return;
        }
        let color_arr: [f32; 4] = [
            color.r.into(),
            color.g.into(),
            color.b.into(),
            color.a.into(),
        ];

        let mut path_builder = Path::builder_with_attributes(4);

        let mut point_iter = points.iter();
        let init_point = point_iter.next().unwrap();
        path_builder.begin(point(init_point.x, init_point.y), &color_arr);

        for current_point in point_iter {
            path_builder.line_to(point(current_point.x, current_point.y), &color_arr);
        }

        path_builder.end(true);

        let path = path_builder.build();
        tessellator.tessellate_path(path);
    }
}
