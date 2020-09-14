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
            let node_borrow = node.borrow();
            let element = node_borrow.as_element().unwrap();
            if element.tag_name() == tag_name {
                return Some((&node, i));
            }
        }
        None
    }

    pub fn pop_until(&mut self, tag_name: &str) {
        while let Some(node) = self.current_node() {
            let node = node.borrow();
            let element = node.as_element().unwrap();
            if element.tag_name() == tag_name {
                self.pop();
                break;
            }
            self.pop();
        }
    }

    pub fn remove_first_matching<F>(&mut self, test: F) where
        F: Fn(&NodeRef) -> bool {
        for (i, node) in self.0.iter().rev().enumerate() {
            if test(node) {
                self.0.remove(i);
                return
            }
        }
    }

    pub fn contains(&self, tag_name: &str) -> bool {
        self.0.iter().any(|node| {
            let node = node.borrow();
            let element = node.as_element().unwrap();
            if element.tag_name() == tag_name {
                return true;
            }
            return false;
        })
    }

    pub fn contains_node(&self, node: NodeRef) -> bool {
        self.0.iter().any(|fnode| *fnode == node)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
