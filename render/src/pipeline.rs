use dom::node::NodePtr;
use gfx::{Bitmap, Canvas};
use layout::{
    formatting_context::{establish_context, FormattingContextType, LayoutContext},
    layout_box::{LayoutBox, LayoutBoxPtr},
};
use painting::Painter;
use shared::{
    primitive::{Rect, Size},
    tree_node::TreeNode,
};
use style_types::ContextualRule;

pub struct Pipeline<'a> {
    painter: Painter<Canvas<'a>>,
}

pub struct PipelineRunOptions {
    pub skip_style_calculation: bool,
}

impl<'a> Pipeline<'a> {
    pub async fn new() -> Pipeline<'a> {
        Pipeline {
            painter: Painter::new(Canvas::new().await),
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
        let layout_node = self.calculate_layout(document_node, size);

        self.painter.resize(size.clone());
        if let Some(node) = layout_node {
            self.painter.paint(&node);
        }
        self.painter.output().await
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
        let layout_tree = layout::tree_builder::TreeBuilder::new().build(document_node);

        if let Some(root) = &layout_tree {
            let layout_context = LayoutContext {
                viewport: Rect {
                    x: 0.,
                    y: 0.,
                    width: size.width,
                    height: size.height,
                },
            };

            let initial_block_box = LayoutBoxPtr(TreeNode::new(LayoutBox::new_anonymous(
                layout::layout_box::BoxData::block_box(),
            )));
            initial_block_box.append_child(root.0.clone());

            establish_context(
                FormattingContextType::BlockFormattingContext,
                initial_block_box.clone(),
            );
            initial_block_box
                .formatting_context()
                .run(&layout_context, initial_block_box.clone());
        }

        layout_tree
    }
}
