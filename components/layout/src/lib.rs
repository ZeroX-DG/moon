pub mod box_model;
pub mod flow;
pub mod formatting_context;
pub mod layout_box;
pub mod tree_builder;
pub mod layout_printer;

use box_model::Rect;
use flow::block::BlockFormattingContext;
use formatting_context::FormattingContext;
use layout_box::LayoutBox;
use style::render_tree::RenderTree;
use tree_builder::TreeBuilder;

pub fn compute_layout(root: &mut LayoutBox, viewport: &Rect) {
    let mut context = BlockFormattingContext::new(viewport);
    context.layout(vec![root], viewport);
}

pub fn build_layout_tree(tree: &RenderTree) -> Option<LayoutBox> {
    let layout_tree_builder = TreeBuilder::new(tree.root.clone().unwrap());

    layout_tree_builder.build()
}
