use super::dom_token_list::DOMTokenList;
use super::node::Node;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::borrow::Cow;

pub struct AttributeMap(HashMap<String, String>);

pub struct Element {
    pub node: Node,
    attributes: AttributeMap,
    id: String,
    class_list: DOMTokenList,
    tag_name: String,
}

impl AttributeMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get_str(&self, attr: &str) -> Cow<str> {
        if let Some(value) = self.0.get(attr) {
            Cow::Borrowed(value)
        } else {
            Cow::Owned(String::new())
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
        write!(f, "Element({:?})", self.tag_name())
    }
}

impl Element {
    pub fn new(tag_name: String) -> Self {
        Self {
            node: Node::new(),
            attributes: AttributeMap::new(),
            id: String::new(),
            class_list: DOMTokenList::new(),
            tag_name,
        }
    }

    pub fn set_attribute(&mut self, name: &str, value: &str) {
        if name == "id" {
            self.id = value.to_string();
            return;
        }
        if name == "class" {
            self.class_list = DOMTokenList::from(value);
            return;
        }
        self.attributes.insert(name.to_owned(), value.to_owned());
    }

    pub fn attributes(&self) -> &AttributeMap {
        &self.attributes
    }

    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    pub fn tag_name(&self) -> String {
        self.tag_name.clone()
    }

    pub fn class_list(&self) -> &DOMTokenList {
        &self.class_list
    }

    pub fn id(&self) -> &String {
        &self.id
    }
}
