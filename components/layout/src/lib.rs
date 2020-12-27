pub mod box_model;
pub mod flow;
pub mod formatting_context;
pub mod layout;
pub mod layout_box;
pub mod tree_builder;

use box_model::Rect;
use layout::LayoutContext;
use layout_box::LayoutBox;
use style::render_tree::RenderTree;
use tree_builder::TreeBuilder;

pub fn compute_layout(root: &mut LayoutBox, viewport: &Rect) {
    layout::layout(
        root,
        viewport,
        &LayoutContext {
            offset_x: 0.,
            offset_y: 0.,
            width: 0.,
            height: 0.,
        },
    );
}

pub fn build_layout_tree(tree: RenderTree) -> Option<LayoutBox> {
    let layout_tree_builder = TreeBuilder::new(tree.root.unwrap());
    
    layout_tree_builder.build()
}
