use dom::node::NodePtr;
use layout::dump_layout;
use layout::formatting_context::{establish_context, FormattingContextType};
use layout::layout_box::LayoutBoxPtr;
use layout::{formatting_context::LayoutContext, layout_box::LayoutBox};
use shared::primitive::{Rect, Size};
use shared::tree_node::TreeNode;
use style_types::ContextualRule;

pub struct Frame {
    document: Option<NodePtr>,
    layout: FrameLayout,
    size: Size,
}

pub struct FrameLayout {
    layout_tree: Option<LayoutBoxPtr>,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            document: None,
            layout: FrameLayout::new(),
            size: Size::new(0., 0.),
        }
    }

    pub fn size(&self) -> Size {
        self.size.clone()
    }

    pub fn resize(&mut self, new_size: Size) {
        self.size = new_size;

        if self.document.is_some() {
            self.layout.reflow(&self.size, self.document(), false);
        }
    }

    pub fn set_document(&mut self, document: NodePtr) {
        self.document = Some(document.clone());
        self.layout.reflow(&self.size, document, true);
    }

    pub fn document(&self) -> NodePtr {
        self.document.clone().expect("No document available")
    }

    pub fn layout(&self) -> &FrameLayout {
        &self.layout
    }
}

impl FrameLayout {
    pub fn new() -> Self {
        Self { layout_tree: None }
    }

    pub fn layout_tree(&self) -> Option<LayoutBoxPtr> {
        self.layout_tree.clone()
    }

    pub fn recalculate_styles(&mut self, document_node: NodePtr) {
        let document = document_node.as_document();
        let style_rules = document.style_rules();

        log::debug!("Begin calculating styles");

        fn compute_styles(element: NodePtr, style_rules: &[ContextualRule]) {
            let computed_styles = style::compute::compute_styles(element.clone(), &style_rules);
            element.set_computed_styles(computed_styles);

            element.for_each_child(|child| compute_styles(NodePtr(child), style_rules))
        }

        compute_styles(document_node, &style_rules);

        log::debug!("Finished calculating styles");
    }

    pub fn recalculate_layout(&mut self, document_node: NodePtr, size: &Size) {
        log::debug!("Building layout tree");
        self.layout_tree = layout::tree_builder::TreeBuilder::new().build(document_node);
        log::debug!("Finished layout tree");

        if let Some(root) = &self.layout_tree {
            log::debug!("Starting layout process");
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
            log::debug!("Finished layout process");
            dump_layout!(root.clone());
        } else {
            log::info!("Empty layout tree. Skipping layout process");
        }
    }

    pub fn reflow(&mut self, size: &Size, document: NodePtr, recalculate_style: bool) {
        log::debug!("Begin reflowing");

        if recalculate_style {
            self.recalculate_styles(document.clone());
        }
        self.recalculate_layout(document, size);

        log::debug!("Finished reflowing");
    }
}
