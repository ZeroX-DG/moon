pub mod box_model;
pub mod flow;
pub mod formatting_context;
pub mod layout_box;
pub mod layout_printer;
pub mod tree_builder;
pub mod line_box;
pub mod line_fragmenter;

use box_model::Rect;
use flow::block::BlockFormattingContext;
use formatting_context::FormattingContext;
use layout_box::LayoutBox;
use style::render_tree::RenderTree;
use tree_builder::TreeBuilder;

pub fn compute_layout(root: &mut LayoutBox, viewport: &Rect) {
    let mut viewport_box = LayoutBox::new_anonymous(layout_box::BoxType::Block);
    viewport_box.box_model().set_width(viewport.width);
    viewport_box.box_model().set_height(viewport.height);
    let mut context = BlockFormattingContext::new(&mut viewport_box);
    context.layout(vec![root]);
}

pub fn build_layout_tree(tree: &RenderTree) -> Option<LayoutBox> {
    let layout_tree_builder = TreeBuilder::new(tree.root.clone().unwrap());

    layout_tree_builder.build()
}
