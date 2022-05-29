use dom::node::NodePtr;
use gfx::Bitmap;
use shared::primitive::Size;

use crate::pipeline::{Pipeline, PipelineRunOptions};

pub struct Frame {
    document: Option<NodePtr>,
    size: Size,
    bitmap: Option<Bitmap>,
}

impl Frame {
    pub fn new(init_size: Size) -> Self {
        Self {
            document: None,
            size: init_size,
            bitmap: None,
        }
    }

    pub fn size(&self) -> Size {
        self.size.clone()
    }

    pub async fn resize(&mut self, new_size: Size, pipeline: &mut Pipeline<'_>) {
        self.size = new_size.clone();
        self.render_frame(
            pipeline,
            PipelineRunOptions {
                skip_style_calculation: true,
            },
        )
        .await;
    }

    pub async fn set_document(&mut self, document: NodePtr, pipeline: &mut Pipeline<'_>) {
        self.document = Some(document.clone());
        self.render_frame(
            pipeline,
            PipelineRunOptions {
                skip_style_calculation: false,
            },
        )
        .await;
    }

    pub fn document(&self) -> Option<NodePtr> {
        self.document.clone()
    }

    pub fn bitmap(&self) -> Option<&Bitmap> {
        self.bitmap.as_ref()
    }

    async fn render_frame(&mut self, pipeline: &mut Pipeline<'_>, opts: PipelineRunOptions) {
        if let Some(document) = self.document() {
            let bitmap = pipeline.run(document, &self.size(), opts).await;
            self.bitmap = Some(bitmap);
        }
    }
}
