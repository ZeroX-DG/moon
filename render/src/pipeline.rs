use dom::node::NodePtr;
use gfx::{Bitmap, Canvas, CanvasCPU, TextMeasure};
use layout::{
    dump_layout,
    formatting_context::{establish_context, FormattingContextType},
    layout_box::{LayoutBox, LayoutBoxPtr},
    layout_context::LayoutContext,
};
use painting::Painter;
use shared::{
    primitive::{Rect, Size},
    tree_node::TreeNode,
};
use style_types::ContextualRule;

pub struct Pipeline {
    painter: Painter,
    layout_tree: Option<LayoutBoxPtr>,
}

pub struct PipelineRunOptions {
    pub skip_style_calculation: bool,
    pub skip_layout_calculation: bool,
}

impl Pipeline {
    pub async fn new() -> Pipeline {
        let painter = if std::env::var("CPU_RENDERING").is_ok() {
            Painter::new(CanvasCPU::new())
        } else {
            Painter::new(Canvas::new().await)
        };
        Pipeline {
            painter,
            layout_tree: None,
        }
    }

    pub async fn run(
        &mut self,
        document_node: NodePtr,
        size: &Size,
        opts: PipelineRunOptions,
    ) -> Bitmap {
        if !opts.skip_style_calculation {
            self.calculate_styles(document_node.clone());
        }

        if !opts.skip_layout_calculation {
            self.layout_tree = self.calculate_layout(document_node, size);
        }

        self.painter.resize(size.clone());
        if let Some(node) = &self.layout_tree {
            self.painter.paint(node);
        }
        self.painter.output().await
    }

    pub fn content(&self) -> Option<LayoutBoxPtr> {
        self.layout_tree.clone()
    }

    fn calculate_styles(&self, document_node: NodePtr) {
        let document = document_node.as_document();
        let style_rules = document.style_rules();

        fn compute_styles(element: NodePtr, style_rules: &[ContextualRule]) {
            let computed_styles = style::compute::compute_styles(element.clone(), &style_rules);
            element.set_computed_styles(computed_styles);

            element.for_each_child(|child| compute_styles(NodePtr(child), style_rules))
        }

        compute_styles(document_node, &style_rules);
    }

    fn calculate_layout(&self, document_node: NodePtr, size: &Size) -> Option<LayoutBoxPtr> {
        let constructed_tree = layout::tree_builder::TreeBuilder::new().build(document_node);
        let mut text_measure = TextMeasure::new();
        let layout_tree = constructed_tree.map(|tree| {
            let mut layout_context = LayoutContext {
                viewport: Rect {
                    x: 0.,
                    y: 0.,
                    width: size.width,
                    height: size.height,
                },
                measure_text_fn: Box::new(move |content, font_size| {
                    text_measure.measure(content, font_size)
                }),
            };

            let initial_block_box = LayoutBoxPtr(TreeNode::new(LayoutBox::new_anonymous(
                layout::layout_box::BoxData::block_box(),
            )));
            initial_block_box.append_child(tree.0.clone());

            establish_context(
                FormattingContextType::BlockFormattingContext,
                initial_block_box.clone(),
            );
            initial_block_box
                .formatting_context()
                .run(&mut layout_context, initial_block_box.clone());

            dump_layout!(initial_block_box);

            initial_block_box
        });

        layout_tree
    }
}
