use super::NodeRef;

const BASE_LIST: [&str; 9] = [
    "applet", "caption", "html", "table", "td", "th", "marquee", "object", "template",
];

#[derive(Debug)]
pub struct StackOfOpenElements(pub Vec<NodeRef>);

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

    pub fn get(&self, index: usize) -> NodeRef {
        return self.0[index].clone();
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

    pub fn remove_first_matching<F>(&mut self, test: F)
    where
        F: Fn(&NodeRef) -> bool,
    {
        for (i, node) in self.0.iter().rev().enumerate() {
            if test(node) {
                self.0.remove(i);
                return;
            }
        }
    }

    pub fn any<F>(&mut self, test: F) -> bool
    where
        F: Fn(&NodeRef) -> bool,
    {
        for node in self.0.iter().rev() {
            if test(node) {
                return true;
            }
        }
        false
    }

    pub fn has_element_name_in_specific_scope(&self, target: &str, list: Vec<&str>) -> bool {
        for node in self.0.iter().rev() {
            let node = node.borrow();
            let element = node.as_element().unwrap();
            if element.tag_name() == target {
                return true;
            }

            if list.contains(&element.tag_name().as_str()) {
                return false;
            }
        }
        return false;
    }

    pub fn has_element_name_in_scope(&self, target: &str) -> bool {
        return self.has_element_name_in_specific_scope(target, BASE_LIST.to_vec());
    }

    pub fn has_element_name_in_button_scope(&self, target: &str) -> bool {
        let mut list = BASE_LIST.to_vec();
        list.push("button");
        return self.has_element_name_in_specific_scope(target, list);
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
