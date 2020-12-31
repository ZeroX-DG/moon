pub mod box_model;
pub mod formatting_context;
pub mod layout;
pub mod layout_box;
pub mod tree_builder;

use box_model::Rect;
use layout_box::LayoutBox;
use style::render_tree::RenderTree;
use tree_builder::TreeBuilder;
use formatting_context::{FormattingContext, FormattingContextType};

pub fn compute_layout(root: &mut LayoutBox, viewport: &Rect) {
    let context = FormattingContext::new(FormattingContextType::Block, root);
    layout::layout(
        root,
        viewport,
        &context.data,
    );
}

pub fn build_layout_tree(tree: RenderTree) -> Option<LayoutBox> {
    let layout_tree_builder = TreeBuilder::new(tree.root.unwrap());
    
    layout_tree_builder.build()
}
