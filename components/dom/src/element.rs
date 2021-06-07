use super::dom_token_list::DOMTokenList;
use super::elements::{ElementData, ElementMethods};
use super::node::NodeHooks;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

pub struct AttributeMap(HashMap<String, String>);

pub struct Element {
    attributes: AttributeMap,
    id: String,
    class_list: DOMTokenList,
    data: ElementData
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
    fn on_inserted(&mut self) {
        self.handle_on_inserted();
    }
}

impl Element {
    pub fn new(data: ElementData) -> Self {
        Self {
            attributes: AttributeMap::new(),
            id: String::new(),
            class_list: DOMTokenList::new(),
            data
        }
    }

    pub fn tag_name(&self) -> String {
        self.data.tag_name()
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
        self.data.handle_attribute_change(name, value);
    }

    pub fn attributes(&self) -> &AttributeMap {
        &self.attributes
    }

    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    pub fn class_list(&self) -> &DOMTokenList {
        &self.class_list
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn handle_on_inserted(&mut self) {
        self.data.handle_on_inserted();
    }
}

