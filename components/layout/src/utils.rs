use dom::dom_ref::NodeRef;

pub fn is_replaced_element(node: &NodeRef) -> bool {
    if let Some(element) = node.borrow().as_element() {
        return match element.tag_name().as_ref() {
            "iframe" | "video" | "img" | "image" | "canvas" => true,
            _ => false,
        };
    }
    false
}

pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
