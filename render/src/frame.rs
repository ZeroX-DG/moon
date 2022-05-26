use dom::node::NodePtr;
use gfx::Bitmap;
use shared::primitive::Size;

use crate::pipeline::{Pipeline, PipelineRunOptions};

pub struct Frame {
    document: Option<NodePtr>,
    size: Size,
    bitmap: Option<Bitmap>
}

impl Frame {
    pub fn new() -> Self {
        Self {
            document: None,
            size: Size::new(0., 0.),
            bitmap: None
        }
    }

    pub fn size(&self) -> Size {
        self.size.clone()
    }

    pub async fn resize(&mut self, new_size: Size, pipeline: &mut Pipeline<'_>) {
        self.size = new_size;
        self.render_frame(pipeline, PipelineRunOptions {
            skip_style_calculation: true
        }).await;
    }

    pub async fn set_document(&mut self, document: NodePtr, pipeline: &mut Pipeline<'_>) {
        self.document = Some(document.clone());
        self.render_frame(pipeline, PipelineRunOptions {
            skip_style_calculation: false
        }).await;
    }

    pub fn document(&self) -> NodePtr {
        self.document.clone().expect("No document available")
    }

    async fn render_frame(&mut self, pipeline: &mut Pipeline<'_>, opts: PipelineRunOptions) {
        let bitmap = pipeline.run(self.document(), &self.size(), opts).await;
        self.bitmap = Some(bitmap);
    }
}

