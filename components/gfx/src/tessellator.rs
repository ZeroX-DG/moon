use lyon_tessellation::{FillTessellator, VertexBuffers, FillOptions, BuffersBuilder, path::Path};

use crate::triangle::{Vertex, Index, VertexConstructor};

pub struct Tessellator {
    fill_tess: FillTessellator,
    vertex_buffers: Vec<VertexBuffers<Vertex, Index>>,
}

impl Tessellator {
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

    pub fn tessellate_path(&mut self, path: Path) {
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

