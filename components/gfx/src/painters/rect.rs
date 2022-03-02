use lyon_tessellation::geom::point;
use lyon_tessellation::path::Path;
use lyon_tessellation::{BuffersBuilder, FillOptions, FillTessellator, VertexBuffers};
use shared::color::Color;
use shared::primitive::{RRect, Rect};

use crate::triangle::{Index, Vertex, VertexConstructor};

pub struct RectPainter {
    fill_tess: FillTessellator,
    vertex_buffers: Vec<VertexBuffers<Vertex, Index>>,
}

impl RectPainter {
    pub fn new() -> Self {
        Self {
            fill_tess: FillTessellator::new(),
            vertex_buffers: Vec::new(),
        }
    }

    pub fn vertex_buffers(&self) -> &[VertexBuffers<Vertex, Index>] {
        &self.vertex_buffers
    }

    pub fn clear(&mut self) {
        self.vertex_buffers.clear();
    }

    pub fn draw_solid_rect(&mut self, rect: &Rect, color: &Color) {
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
        self.tessellate_path(path);
    }

    pub fn draw_solid_rrect(&mut self, rect: &RRect, color: &Color) {
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
        self.tessellate_path(path);
    }

    fn tessellate_path(&mut self, path: Path) {
        let mut buffer: VertexBuffers<Vertex, Index> = VertexBuffers::new();

        let result = self.fill_tess.tessellate_with_ids(
            path.id_iter(),
            &path,
            Some(&path),
            &FillOptions::DEFAULT,
            &mut BuffersBuilder::new(&mut buffer, VertexConstructor),
        );

        if let Err(e) = result {
            log::error!("Tessellation failed: {:?}", e);
            return;
        }

        self.vertex_buffers.push(buffer);
    }
}
