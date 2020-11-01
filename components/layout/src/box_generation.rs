use super::layout_box::{BoxType, FormattingContext, LayoutBox};
use style::render_tree::RenderNodeRef;
use style::value_processing::{Property, Value};
use style::values::display::Display;

/// Box generation for layout
/// https://www.w3.org/TR/CSS22/visuren.html#box-gen
pub fn generate_box(root: RenderNodeRef) -> Option<LayoutBox> {
    if root.borrow().node.is::<dom::text::Text>() {
        return Some(LayoutBox::new(root.clone(), BoxType::Anonymous));
    }

    let display = root.borrow().get_style(&Property::Display);

    let mut layout_box = match **display {
        Value::Display(Display::Block) => LayoutBox::new(root.clone(), BoxType::Block),
        Value::Display(Display::Inline) => LayoutBox::new(root.clone(), BoxType::Inline),
        _ => return None,
    };

    for child in &root.borrow().children {
        if let Some(child_box) = generate_box(child.clone()) {
            if layout_box.fmt_context.is_none() {
                match child_box.box_type {
                    BoxType::Block => {
                        layout_box.set_formatting_context(FormattingContext::Block);
                    }
                    _ => {}
                }
            }

            layout_box.add_child(child_box)
        }
    }

    if layout_box.fmt_context.is_none() {
        layout_box.set_formatting_context(FormattingContext::Inline);
    }

    let layout_box_fmt_context = layout_box.fmt_context.clone().unwrap();

    for child in layout_box.children.iter_mut() {
        child.set_parent_formatting_context(layout_box_fmt_context.clone());
    }

    Some(layout_box)
}

/// Wrap inline boxes in anonymous box when they have been
/// broken in block formatting context
pub fn wrap_inline_boxes(root: &mut LayoutBox) {
    if let Some(FormattingContext::Block) = root.fmt_context {
        let mut is_block_start = false;

        root.children = root
            .children
            .clone()
            .into_iter()
            .fold(vec![], |mut acc, current| match current.box_type {
                BoxType::Block | BoxType::Anonymous => {
                    is_block_start = false;
                    acc.push(current);
                    acc
                }
                BoxType::Inline => {
                    let can_append = if let Some(last_box) = acc.last() {
                        match last_box.box_type {
                            BoxType::Anonymous if is_block_start => true,
                            _ => false,
                        }
                    } else {
                        false
                    };

                    if can_append {
                        if let Some(last_box) = acc.last_mut() {
                            last_box.add_child(current);
                        }
                    } else {
                        is_block_start = true;
                        let mut contain_box =
                            LayoutBox::new(root.render_node.clone(), BoxType::Anonymous);
                        contain_box.set_formatting_context(FormattingContext::Inline);
                        contain_box.add_child(current);
                        acc.push(contain_box);
                    }
                    acc
                }
            });
    }

    for child in root.children.iter_mut() {
        wrap_inline_boxes(child);
    }
}
