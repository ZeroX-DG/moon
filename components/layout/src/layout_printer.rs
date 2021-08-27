use super::layout_box::LayoutNode;

pub enum DumpSpecificity {
    Structure,
    StructureAndDimensions,
}

pub fn dump_layout(root: &LayoutNode) {
    println!("{}", layout_to_string(root, 0, &DumpSpecificity::Structure));
}

pub fn layout_to_string(root: &LayoutNode, level: usize, specificity: &DumpSpecificity) -> String {
    let mut result = String::new();
    let child_nodes = root.children();

    let box_type = if root.is_anonymous() {
        format!("[Anonymous {}]", root.friendly_name())
    } else {
        format!("[{}]", root.friendly_name())
    };

    let dimensions = match specificity {
        DumpSpecificity::Structure => String::new(),
        DumpSpecificity::StructureAndDimensions => format!(
            " (x: {} | y: {} | w: {} | h: {})",
            root.dimensions().content.x,
            root.dimensions().content.y,
            root.dimensions().content.width,
            root.dimensions().content.height
        ),
    };

    let node_info = match &root.render_node() {
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
        result.push_str(&layout_to_string(node, level + 1, specificity));
    }
    return result;
}
