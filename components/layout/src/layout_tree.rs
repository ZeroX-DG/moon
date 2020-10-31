use style::render_tree::RenderNodeRef;
use style::value_processing::{Property, Value};
use style::values::display::Display;

/// LayoutBox for the layout tree
#[derive(Debug, Clone)]
pub struct LayoutBox {
    pub box_type: BoxType,
    pub render_node: Option<RenderNodeRef>,
    pub dimensions: Dimensions,
    pub children: Vec<LayoutBox>,
    pub fmt_context: Option<FormattingContext>
}

/// Formatting context of each box
#[derive(Debug, Clone, PartialEq)]
pub enum FormattingContext {
    Block,
    Inline
}

/// Different box types for each layout box
#[derive(Debug, Clone, PartialEq)]
pub enum BoxType {
    Block,
    Inline,
    Anonymous
}

/// Box-model dimensions for each layout box
#[derive(Debug, Clone)]
pub struct Dimensions {
    pub content: ContentSize,
    pub padding: EdgeSizes,
    pub margin: EdgeSizes,
    pub border: EdgeSizes
}

/// Size of the content area (all in px)
#[derive(Debug, Clone)]
pub struct ContentSize {
    pub width: f32,
    pub height: f32
}

/// Edge size of the box (all in px)
#[derive(Debug, Clone)]
pub struct EdgeSizes {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32
}

impl Default for Dimensions {
    fn default() -> Self {
        Self {
            content: ContentSize {
                width: 0.0,
                height: 0.0
            },
            padding: Default::default(),
            border: Default::default(),
            margin: Default::default()
        }
    }
}

impl Default for EdgeSizes {
    fn default() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0 
        }
    }
}

impl LayoutBox {
    pub fn new(node: Option<RenderNodeRef>, box_type: BoxType) -> Self {
        Self {
            box_type,
            render_node: node,
            dimensions: Dimensions::default(),
            children: Vec::new(),
            fmt_context: None
        }
    }

    pub fn add_child(&mut self, child: LayoutBox) {
        self.children.push(child);
    }

    pub fn set_formatting_context(&mut self, ctx: FormattingContext) {
        self.fmt_context = Some(ctx);
    }
}

/// Build the layout tree from root render node
///
/// There are 2 steps to this process
/// 1. **Generate tree:** generate layout boxes for elements
/// 2. **Wrap inline boxes:** when block-level box breaks the inline boxes
/// this process wrap those inline boxes into anonymous block box
pub fn build_layout_tree(root: RenderNodeRef) -> Option<LayoutBox> {
    let mut root_box = generate_box(root.clone());
    if let Some(b) = &mut root_box {
        wrap_inline_boxes(b);
    }
    root_box
}

/// Box generation for layout
/// https://www.w3.org/TR/CSS22/visuren.html#box-gen
fn generate_box(root: RenderNodeRef) -> Option<LayoutBox> {
    if root.borrow().node.is::<dom::text::Text>() {
        return Some(LayoutBox::new(
            Some(root.clone()),
            BoxType::Anonymous,
        ));
    }

    let display = root.borrow().get_style(&Property::Display);

    let mut layout_box = match **display {
        Value::Display(Display::Block) => LayoutBox::new(
            Some(root.clone()),
            BoxType::Block,
        ),
        Value::Display(Display::Inline) => LayoutBox::new(
            Some(root.clone()),
            BoxType::Inline,
        ),
        _ => return None
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

    Some(layout_box)
}

/// Wrap inline boxes in anonymous box when they have been
/// broken in block formatting context
fn wrap_inline_boxes(root: &mut LayoutBox) {
    if let Some(FormattingContext::Block) = root.fmt_context {
        let mut is_block_start = false;

        root.children = root.children
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
                            _ => false
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
                        let mut contain_box = LayoutBox::new(None, BoxType::Anonymous);
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

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::css::parse_stylesheet;
    use test_utils::dom_creator::*;
    use css::cssom::css_rule::CSSRule;
    use style::value_processing::*;
    use style::render_tree::build_render_tree;
    use dom::dom_ref::NodeRef;

    fn print_tree(root: NodeRef, level: usize) {
        let child_nodes = root.borrow().as_node().child_nodes();
        println!(
            "{}{:#?}({} child)",
            "    ".repeat(level),
            root,
            child_nodes.length()
        );
        for node in child_nodes {
            print_tree(node, level + 1);
        }
    }

    #[test]
    fn test_build_tree() {
        let dom = element("div", vec![
            element("span", vec![text("hello")]),
            element("p", vec![text("world")]),
            element("span", vec![text("hello")]),
            element("span", vec![text("hello")]),
        ]);

        print_tree(dom.clone(), 0);

        let css = r#"
        div {
            display: block;
        }
        p {
            display: block;
        }
        span {
            display: inline;
        }
        "#;

        let stylesheet = parse_stylesheet(css);

        let rules = stylesheet
            .iter()
            .map(|rule| match rule {
                CSSRule::Style(style) => ContextualRule {
                    inner: style,
                    location: CSSLocation::Embedded,
                    origin: CascadeOrigin::User,
                },
            })
            .collect::<Vec<ContextualRule>>();

        let render_tree = build_render_tree(dom.clone(), &rules);
        let layout_tree = build_layout_tree(render_tree.root.unwrap()).unwrap();

        assert_eq!(layout_tree.box_type, BoxType::Block);
        assert_eq!(layout_tree.fmt_context, Some(FormattingContext::Block));
        // span
        assert_eq!(layout_tree.children[0].box_type, BoxType::Anonymous);
        assert_eq!(layout_tree.children[0].fmt_context, Some(FormattingContext::Inline));
        assert_eq!(layout_tree.children[0].children[0].box_type, BoxType::Inline);
        // p
        assert_eq!(layout_tree.children[1].box_type, BoxType::Block);
        assert_eq!(layout_tree.children[1].fmt_context, Some(FormattingContext::Inline));
        assert_eq!(layout_tree.children[1].children[0].box_type, BoxType::Anonymous);
        // last 2 span is grouped
        assert_eq!(layout_tree.children[2].box_type, BoxType::Anonymous);
        assert_eq!(layout_tree.children[2].fmt_context, Some(FormattingContext::Inline));
        assert_eq!(layout_tree.children[2].children[0].box_type, BoxType::Inline);
        assert_eq!(layout_tree.children[2].children[1].box_type, BoxType::Inline);
    }
}
