use dom::node::Node;

use super::Element;
use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};

const BASE_LIST: [&str; 9] = [
    "applet", "caption", "html", "table", "td", "th", "marquee", "object", "template",
];

#[derive(Debug)]
pub struct StackOfOpenElements(pub Vec<Rc<Node>>);

impl Deref for StackOfOpenElements {
    type Target = Vec<Rc<Node>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StackOfOpenElements {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl StackOfOpenElements {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn current_node(&self) -> Option<Rc<Node>> {
        if let Some(node) = self.0.last() {
            return Some(node.clone());
        }
        None
    }

    pub fn get(&self, index: usize) -> Rc<Node> {
        return self.0[index].clone();
    }

    pub fn last_element_with_tag_name(&self, tag_name: &str) -> Option<(&Rc<Node>, usize)> {
        for (i, node) in self.0.iter().rev().enumerate() {
            let element = node.as_element();
            if element.tag_name() == tag_name {
                return Some((&node, i));
            }
        }
        None
    }

    pub fn pop_until(&mut self, tag_name: &str) {
        while let Some(node) = self.current_node() {
            let element = node.as_element();
            if element.tag_name() == tag_name {
                self.0.pop();
                break;
            }
            self.0.pop();
        }
    }

    pub fn pop_until_match<F>(&mut self, test: F)
    where
        F: Fn(&Element) -> bool,
    {
        while let Some(node) = self.current_node() {
            let element = node.as_element();
            if test(element) {
                self.0.pop();
                break;
            }
            self.0.pop();
        }
    }

    pub fn clear_back_to_table_context(&mut self) {
        while let Some(node) = self.current_node() {
            let element = node.as_element();
            let element_tag_name = element.tag_name();
            if element_tag_name == "table"
                || element_tag_name == "template"
                || element_tag_name == "html"
            {
                break;
            }
            self.0.pop();
        }
    }

    pub fn remove_first_matching<F>(&mut self, test: F)
    where
        F: Fn(&Rc<Node>) -> bool,
    {
        for (i, node) in self.0.iter().rev().enumerate() {
            if test(node) {
                self.0.remove(i);
                return;
            }
        }
    }

    pub fn any<F>(&self, test: F) -> bool
    where
        F: Fn(&Rc<Node>) -> bool,
    {
        self.0.iter().any(test)
    }

    pub fn has_element_name_in_specific_scope(&self, target: &str, list: Vec<&str>) -> bool {
        for node in self.0.iter().rev() {
            let element = node.as_element();
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

    pub fn has_element_name_in_list_item_scope(&self, target: &str) -> bool {
        let mut list = BASE_LIST.to_vec();
        list.push("ol");
        list.push("ul");
        return self.has_element_name_in_specific_scope(target, list);
    }

    pub fn has_element_name_in_table_scope(&self, target: &str) -> bool {
        let mut list = BASE_LIST.to_vec();
        list.push("html");
        list.push("table");
        list.push("template");
        return self.has_element_name_in_specific_scope(target, list);
    }

    pub fn has_element_name_in_select_scope(&self, target: &str) -> bool {
        let list = BASE_LIST
            .to_vec()
            .iter()
            .filter(|item| **item != "option" || **item != "optgroup")
            .map(|item| *item)
            .collect();
        return self.has_element_name_in_specific_scope(target, list);
    }

    pub fn has_element_in_specific_scope(&self, target: &Rc<Node>, list: Vec<&str>) -> bool {
        for node in self.0.iter().rev() {
            if Rc::ptr_eq(node, target) {
                return true;
            }

            let element = node.as_element();

            if list.contains(&element.tag_name().as_str()) {
                return false;
            }
        }
        return false;
    }

    pub fn has_element_in_scope(&self, target: &Rc<Node>) -> bool {
        self.has_element_in_specific_scope(target, BASE_LIST.to_vec())
    }

    pub fn contains(&self, tag_name: &str) -> bool {
        self.any(|node| {
            let element = node.as_element();
            if element.tag_name() == tag_name {
                return true;
            }
            return false;
        })
    }

    pub fn contains_node(&self, node: &Rc<Node>) -> bool {
        self.any(|fnode| Rc::ptr_eq(fnode, node))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
