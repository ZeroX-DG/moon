use style::render_tree::RenderNodeRef;
use style::value_processing::{Property, Value};
use style::values::display::Display;

/// LayoutBox for the layout tree
pub struct LayoutBox {
    pub box_type: BoxType,
    pub render_node: RenderNodeRef,
    pub dimensions: Dimensions,
    pub children: Vec<LayoutBox>,
}

/// Different box types for each layout box
pub enum BoxType {
    Block,
    Inline,
    Anonymous,
    AnonymousInline
}

/// Box-model dimensions for each layout box
pub struct Dimensions {
    pub content: ContentSize,
    pub padding: EdgeSizes,
    pub margin: EdgeSizes,
    pub border: EdgeSizes
}

/// Size of the content area (all in px)
pub struct ContentSize {
    pub width: f32,
    pub height: f32
}

/// Edge size of the box (all in px)
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
    pub fn new(node: RenderNodeRef, box_type: BoxType) -> Self {
        Self {
            box_type,
            render_node: node,
            dimensions: Dimensions::default(),
            children: Vec::new()
        }
    }

    pub fn add_child(&mut self, child: LayoutBox) {
        self.children.push(child);
    }
}

/// Box generation for layout
/// https://www.w3.org/TR/CSS22/visuren.html#box-gen
pub fn generate_box(root: RenderNodeRef) -> Option<LayoutBox> {
    // TODO: careful with text nodes, they don't have a style and may panic
    let display = root.borrow().get_style(&Property::Display);

    let mut layout_box = match **display {
        Value::Display(Display::Block) => LayoutBox::new(root.clone(), BoxType::Block),
        Value::Display(Display::Inline) => LayoutBox::new(root.clone(), BoxType::Inline),
        _ => return None
    };

    for child in &root.borrow().children {
        if let Some(child_box) = generate_box(child.clone()) {
            match child_box.box_type {
                BoxType::Block => layout_box.add_child(child_box),
                BoxType::Inline => layout_box.add_child(child_box),
                _ => {}
            }
        }
    }

    Some(layout_box)
}
