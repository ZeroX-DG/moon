use std::{fmt::Debug, ptr::NonNull};
use std::any::Any;
use std::convert::{AsRef, AsMut};

use style::render_tree::RenderNodeRef;

#[derive(Debug, Clone)]
pub struct LayoutNodePtr(NonNull<LayoutNode>);

pub trait LayoutBox: Any + Debug {
    fn is_block(&self) -> bool;
    fn is_inline(&self) -> bool;
    fn render_node(&self) -> Option<RenderNodeRef>;
    fn is_anonymous(&self) -> bool {
        self.render_node().is_none()
    }
}

#[derive(Debug)]
pub struct LayoutNode {
    inner: Box<dyn LayoutBox>,
    parent: Option<LayoutNodePtr>,
    children: Vec<LayoutNode>
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
        Self(NonNull::new(layout_node_ref).expect("Unable to create node pointer"))
    }
}

impl std::ops::Deref for LayoutNode {
    type Target = Box<dyn LayoutBox>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl LayoutNode {
    pub fn new<B: LayoutBox>(layout_box: B) -> Self {
        Self {
            inner: Box::new(layout_box),
            parent: None,
            children: Vec::new()
        }
    }

    pub fn new_boxed(layout_box: Box<dyn LayoutBox>) -> Self {
        Self {
            inner: layout_box,
            parent: None,
            children: Vec::new()
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

    pub fn add_child(&mut self, child: LayoutNode) {
        self.children.push(child);
    }
}
