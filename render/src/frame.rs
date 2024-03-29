use dom::node::NodePtr;
use gfx::Bitmap;
use layout::layout_box::LayoutBoxPtr;
use shared::primitive::{Point, Size};

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

    pub async fn resize(&mut self, new_size: Size, pipeline: &mut Pipeline) {
        self.size = new_size.clone();
        self.render_frame(
            pipeline,
            PipelineRunOptions {
                skip_style_calculation: true,
                skip_layout_calculation: false,
            },
        )
        .await;
    }

    pub async fn handle_mouse_move(&self, coord: Point, pipeline: &mut Pipeline) {
        if let Some(root_node) = pipeline.content() {
            root_node.handle_mouse_move(&coord);
        }

        // TODO: Re-render the frame if needed
    }

    pub async fn scroll(&mut self, delta_y: f32, pipeline: &mut Pipeline) {
        let mut need_redraw = false;

        // TODO: Handle scrolling for other overflow element within the current frame's document
        if let Some(root_node) = pipeline.content() {
            let deepest_scrollable_container = root_node.find_first_deepest_decendant(|node| {
                let node = LayoutBoxPtr(node);
                node.is_mouse_over() && node.scrollable()
            });

            if let Some(node) = deepest_scrollable_container {
                need_redraw = LayoutBoxPtr(node).scroll(delta_y);
            } else {
                need_redraw = root_node.scroll(delta_y);
            }
        }

        if !need_redraw {
            return;
        }

        self.render_frame(
            pipeline,
            PipelineRunOptions {
                skip_style_calculation: true,
                skip_layout_calculation: true,
            },
        )
        .await;
    }

    pub async fn set_document(&mut self, document: NodePtr, pipeline: &mut Pipeline) {
        self.document = Some(document.clone());
        self.render_frame(
            pipeline,
            PipelineRunOptions {
                skip_style_calculation: false,
                skip_layout_calculation: false,
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

    async fn render_frame(&mut self, pipeline: &mut Pipeline, opts: PipelineRunOptions) {
        if let Some(document) = self.document() {
            let bitmap = pipeline.run(document, &self.size(), opts).await;
            self.bitmap = Some(bitmap);
        }
    }
}
