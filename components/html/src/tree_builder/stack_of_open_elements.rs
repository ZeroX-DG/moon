use super::NodeRef;

#[derive(Debug)]
pub struct StackOfOpenElements(Vec<NodeRef>);

impl StackOfOpenElements {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, node: NodeRef) {
        self.0.push(node)
    }

    pub fn pop(&mut self) -> Option<NodeRef> {
        self.0.pop()
    }

    pub fn current_node(&self) -> Option<NodeRef> {
        if let Some(node) = self.0.last() {
            return Some(node.clone());
        }
        None
    }

    pub fn last_element_with_tag_name(&self, tag_name: &str) -> Option<(&NodeRef, usize)> {
        for (i, node) in self.0.iter().rev().enumerate() {
            let element = node.borrow().as_element().unwrap();
            if element.tag_name() == tag_name {
                return Some((&node, i));
            }
        }
        None
    }

    pub fn pop_until(&mut self, tag_name: &str) {
        while let Some(node) = self.current_node() {
            let element = node.borrow().as_element().unwrap();
            if element.tag_name() == tag_name {
                self.pop();
                break;
            }
            self.pop();
        }
    }

    pub fn contains(&self, tag_name: &str) -> bool {
        self.0.iter().any(|node| {
            let element = node.borrow().as_element().unwrap();
            if element.tag_name() == tag_name {
                return true;
            }
            return false;
        })
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
