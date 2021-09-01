use crate::layout_box::{LayoutNodeId, LayoutTree};

pub enum DumpSpecificity {
    Structure,
    StructureAndDimensions,
}

pub fn dump_layout(tree: &LayoutTree, root: &LayoutNodeId) {
    println!("{}", layout_to_string(tree, root, 0, &DumpSpecificity::StructureAndDimensions));
}

pub fn layout_to_string(tree: &LayoutTree, root: &LayoutNodeId, level: usize, specificity: &DumpSpecificity) -> String {
    let mut result = String::new();
    let child_nodes = tree.children(root);

    let root_node = tree.get_node(root);

    let box_type = if root_node.is_anonymous() {
        format!("[{}][Anonymous {}]", root, root_node.friendly_name())
    } else {
        format!("[{}][{}]", root, root_node.friendly_name())
    };

    let dimensions = match specificity {
        DumpSpecificity::Structure => String::new(),
        DumpSpecificity::StructureAndDimensions => format!(
            " (x: {} | y: {} | w: {} | h: {})",
            root_node.dimensions().content.x,
            root_node.dimensions().content.y,
            root_node.dimensions().content.width,
            root_node.dimensions().content.height
        ),
    };

    let node_info = match &root_node.render_node() {
        Some(node) => format!(" {:#?}", node.borrow().node),
        None => String::new(),
    };

    result.push_str(&format!(
        "{}{}{}{}\n",
        "  ".repeat(level),
        box_type,
        node_info,
        dimensions
    ));

    for node in child_nodes {
        result.push_str(&layout_to_string(tree, node, level + 1, specificity));
    }
    return result;
}
