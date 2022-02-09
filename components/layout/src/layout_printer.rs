use std::rc::Rc;

use crate::{
    flow::line_box::{LineBox, LineFragment, LineFragmentData},
    layout_box::LayoutBox,
};

pub enum DumpSpecificity {
    Structure,
    All,
}

#[macro_export]
macro_rules! dump_layout {
    ($node:expr) => {
        use layout::layout_printer::{layout_to_string, DumpSpecificity};
        layout_to_string($node, 0, &DumpSpecificity::All)
            .lines()
            .for_each(|line| log::debug!("{}", line));
    };
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
        DumpSpecificity::All => format!(
            " (x: {} | y: {} | w: {} | h: {})",
            root_node.absolute_rect().x,
            root_node.absolute_rect().y,
            root_node.absolute_rect().width,
            root_node.absolute_rect().height,
        ),
    };

    let node_info = match &root_node.render_node() {
        Some(node) => format!(" {:?}", node.node),
        None => String::new(),
    };

    result.push_str(&format!(
        "{}{}{}{}\n",
        "  ".repeat(level),
        box_type,
        node_info,
        dimensions
    ));

    if root_node.is_block() && root_node.children_are_inline() {
        for line in root_node.lines().borrow().iter() {
            result.push_str(&line_box_to_string(line, level + 1, specificity));
        }
    } else {
        for node in root_node.children().iter() {
            result.push_str(&layout_to_string(node.clone(), level + 1, specificity));
        }
    }

    return result;
}

fn line_box_to_string(line: &LineBox, level: usize, specificity: &DumpSpecificity) -> String {
    let mut result = String::new();

    let line_dimensions = format!("(w: {} | h: {})", line.size.width, line.size.height);

    result.push_str(&format!("{}[LineBox]{}\n", "  ".repeat(level), line_dimensions));

    for fragment in line.fragments.iter() {
        result.push_str(&fragment_to_string(fragment, level + 1, specificity));
    }

    result
}

fn fragment_to_string(
    fragment: &LineFragment,
    level: usize,
    specificity: &DumpSpecificity,
) -> String {
    let fragment_type = match fragment.data {
        LineFragmentData::Box(_) => "[Box Fragment]",
    };

    let fragment_info = format!(
        "(x: {} | y: {} | w: {} | h: {})",
        fragment.offset.x, fragment.offset.y, fragment.size.width, fragment.size.height
    );

    let mut result = format!("{}{}{}\n", "  ".repeat(level), fragment_type, fragment_info);
    match &fragment.data {
        LineFragmentData::Box(node) => {
            result.push_str(&layout_to_string(node.clone(), level + 1, specificity))
        }
    }
    result
}
