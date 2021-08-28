use std::{
    any::Any,
    convert::{AsMut, AsRef},
    ptr::NonNull,
    fmt::Debug,
};

use style::{render_tree::RenderNodeRef, value_processing::{Property, Value}, values::position::Position};

use crate::box_model::Dimensions;

#[derive(Debug, Clone)]
pub struct LayoutNodePtr(NonNull<LayoutNode>);

pub trait LayoutBox: Any + Debug {
    fn is_block(&self) -> bool;
    fn is_inline(&self) -> bool;
    fn render_node(&self) -> Option<RenderNodeRef>;
    fn is_anonymous(&self) -> bool {
        self.render_node().is_none()
    }
    fn friendly_name(&self) -> &str;
    fn dimensions(&self) -> &Dimensions;
    fn dimensions_mut(&mut self) -> &mut Dimensions;
    fn is_positioned(&self, position: Position) -> bool {
        match self.render_node() {
            Some(node) => match node.borrow().get_style(&Property::Position).inner() {
                Value::Position(pos) => *pos == position,
                _ => false
            }
            _ => false
        }
    }
    fn is_non_replaced(&self) -> bool {
        match &self.render_node() {
            Some(node) => match node.borrow().node.borrow().as_element_opt() {
                Some(e) => match e.tag_name().as_str() {
                    "video" | "image" | "img" | "canvas" => false,
                    _ => true,
                },
                _ => true,
            },
            _ => true,
        }
    }
    fn is_style_auto(&self, property: &Property) -> bool {
        if let Some(node) = &self.render_node() {
            let style = node.borrow().get_style(property);

            return style.is_auto();
        }
        return true;
    }
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Debug)]
pub struct LayoutNode {
    inner: Box<dyn LayoutBox>,
    parent: Option<LayoutNodePtr>,
    children: Vec<LayoutNode>,
}

impl AsRef<LayoutNode> for LayoutNodePtr {
    fn as_ref(&self) -> &LayoutNode {
        unsafe { self.0.as_ref() }
    }
}

impl AsMut<LayoutNode> for LayoutNodePtr {
    fn as_mut(&mut self) -> &mut LayoutNode {
        unsafe { self.0.as_mut() }
    }
}

impl From<&mut LayoutNode> for LayoutNodePtr {
    fn from(layout_node_ref: &mut LayoutNode) -> Self {
        unsafe { Self(NonNull::new_unchecked(layout_node_ref)) }
    }
}

impl std::ops::Deref for LayoutNode {
    type Target = Box<dyn LayoutBox>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for LayoutNode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl LayoutNode {
    pub fn new<B: LayoutBox>(layout_box: B) -> Self {
        Self {
            inner: Box::new(layout_box),
            parent: None,
            children: Vec::new(),
        }
    }

    pub fn new_boxed(layout_box: Box<dyn LayoutBox>) -> Self {
        Self {
            inner: layout_box,
            parent: None,
            children: Vec::new(),
        }
    }

    pub fn children(&self) -> &[LayoutNode] {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut Vec<LayoutNode> {
        &mut self.children
    }

    pub fn children_are_inline(&self) -> bool {
        self.children.iter().all(|child| child.as_ref().is_inline())
    }

    pub fn set_children(&mut self, children: Vec<LayoutNode>) {
        self.children = children;
    }

    pub fn set_parent(&mut self, parent: &mut LayoutNode) {
        self.parent = Some(LayoutNodePtr::from(parent));
    }

    pub fn add_child(&mut self, mut child: LayoutNode) {
        child.set_parent(self);
        self.children.push(child);
    }

    pub fn as_box<B: LayoutBox>(&self) -> &B {
        self.inner.as_any().downcast_ref::<B>().expect("Invalid box casting")
    }

    pub fn as_box_mut<B: LayoutBox>(&mut self) -> &mut B {
        self.inner.as_any_mut().downcast_mut::<B>().expect("Invalid box casting")
    }

    pub fn containing_block(&self) -> LayoutNodePtr {
    }
}
