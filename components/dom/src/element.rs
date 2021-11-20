use crate::node::Node;

use super::dom_token_list::DOMTokenList;
use super::elements::{ElementData, ElementMethods};
use super::node::NodeHooks;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

#[derive(Clone)]
pub struct AttributeMap(HashMap<String, String>);

pub struct Element {
    attributes: RefCell<AttributeMap>,
    id: RefCell<Option<String>>,
    class_list: RefCell<DOMTokenList>,
    data: ElementData,
}

impl AttributeMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get_str(&self, attr: &str) -> String {
        if let Some(value) = self.0.get(attr) {
            value.to_string()
        } else {
            String::new()
        }
    }

    pub fn get_bool(&self, attr: &str) -> bool {
        if let Some(value) = self.0.get(attr) {
            value.is_empty() || value.to_lowercase() == attr.to_lowercase()
        } else {
            false
        }
    }
}

impl Deref for AttributeMap {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AttributeMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl core::fmt::Debug for Element {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Element({:?})", self.data)
    }
}

impl NodeHooks for Element {
    fn on_inserted(&self, document: Rc<Node>) {
        self.handle_on_inserted(document);
    }
}

impl Element {
    pub fn new(data: ElementData) -> Self {
        Self {
            attributes: RefCell::new(AttributeMap::new()),
            id: RefCell::new(None),
            class_list: RefCell::new(DOMTokenList::new()),
            data,
        }
    }

    pub fn tag_name(&self) -> String {
        self.data.tag_name()
    }

    pub fn set_attribute(&self, name: &str, value: &str) {
        if name == "id" {
            *self.id.borrow_mut() = Some(value.to_string());
            return;
        }
        if name == "class" {
            *self.class_list.borrow_mut() = DOMTokenList::from(value);
            return;
        }
        self.attributes
            .borrow_mut()
            .insert(name.to_owned(), value.to_owned());
        self.data.handle_attribute_change(name, value);
    }

    pub fn attributes(&self) -> RefCell<AttributeMap> {
        self.attributes.clone()
    }

    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.borrow().contains_key(name)
    }

    pub fn class_list(&self) -> RefCell<DOMTokenList> {
        self.class_list.clone()
    }

    pub fn id(&self) -> Option<String> {
        self.id.borrow().clone()
    }

    pub fn handle_on_inserted(&self, document: Rc<Node>) {
        self.data.handle_on_inserted(document);
    }
}
