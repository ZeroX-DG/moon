use lyon_tessellation::geom::point;
use lyon_tessellation::path::Path;
use lyon_tessellation::{BuffersBuilder, FillOptions, FillTessellator, VertexBuffers};
use painting::{Color, RRect, Rect};

use crate::triangle::{Vertex, VertexConstructor, Index};

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

    pub fn draw_solid_rect(&mut self, rect: &Rect, color: &Color) {
        let mut buffer: VertexBuffers<Vertex, Index> = VertexBuffers::new();

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

    pub fn draw_solid_rrect(&mut self, rect: &RRect, color: &Color) {
        self.draw_solid_rect(&Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height
        }, color);
    }
}

