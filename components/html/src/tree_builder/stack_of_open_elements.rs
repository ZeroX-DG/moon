use super::NodeRef;
use dom::implementations::Element;

pub struct StackOfOpenElements(Vec<NodeRef>);

impl StackOfOpenElements {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, node: NodeRef) {
        self.0.push(node)
    }

    pub fn current_node(&self) -> Option<NodeRef> {
        if let Some(node) = self.0.last() {
            return Some(node.clone());
        }
        None
    }

    pub fn last_element_with_tag_name(&self, tag_name: &str) -> Option<(&NodeRef, usize)> {
        for (i, node) in self.0.iter().rev().enumerate() {
            if node.tag_name() == tag_name {
                return Some((&node, i))
            }
        }
        None
    }
}
