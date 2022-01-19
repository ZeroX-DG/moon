use std::rc::Rc;

use crate::layout_box::LayoutBox;

pub enum DumpSpecificity {
    Structure,
    StructureAndDimensions,
}

pub fn dump_layout(root: Rc<LayoutBox>) {
    println!(
        "{}",
        layout_to_string(root, 0, &DumpSpecificity::StructureAndDimensions)
    );
}

pub fn layout_to_string(
    root_node: Rc<LayoutBox>,
    level: usize,
    specificity: &DumpSpecificity,
) -> String {
    let mut result = String::new();

    let box_type = if root_node.is_anonymous() {
        format!("[Anonymous {}]", root_node.friendly_name())
    } else {
        format!("[{}]", root_node.friendly_name())
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
        Some(node) => format!(" {:#?}", node.node),
        None => String::new(),
    };

    result.push_str(&format!(
        "{}{}{}{}\n",
        "  ".repeat(level),
        box_type,
        node_info,
        dimensions
    ));

    for node in root_node.children().iter() {
        result.push_str(&layout_to_string(node.clone(), level + 1, specificity));
    }
    return result;
}
