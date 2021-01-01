use super::layout_box::LayoutBox;
use std::io::Write;

pub enum DumpSpecificity {
    Structure,
    StructureAndDimensions,
}

pub fn layout_to_string(root: &LayoutBox, level: usize, specificity: &DumpSpecificity) -> String {
    let mut result = String::new();
    let child_nodes = &root.children;

    let box_type = if root.is_anonymous() {
        format!("[Anonymous {:?}]", root.box_type)
    } else {
        format!("[{:?}]", root.box_type)
    };

    let dimensions = match specificity {
        DumpSpecificity::Structure => String::new(),
        DumpSpecificity::StructureAndDimensions => format!(
            " (x: {} | y: {} | w: {} | h: {})",
            root.dimensions.content.x,
            root.dimensions.content.y,
            root.dimensions.content.width,
            root.dimensions.content.height
        ),
    };

    let node_info = match &root.render_node {
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

pub fn dump_layout<W: Write>(
    root: &LayoutBox,
    specificity: &DumpSpecificity,
    output: &mut W,
) -> std::io::Result<()> {
    let result = layout_to_string(root, 0, specificity);

    output.write_all(result.as_bytes())
}
