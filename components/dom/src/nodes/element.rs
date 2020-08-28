use crate::dom_token_list::DOMTokenList;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use super::HTMLElement;

pub enum ElementData {
    HTMLElement(HTMLElement)
}

pub struct Element {
    pub(crate) data: ElementData,
    pub(crate) attributes: HashMap<String, String>,
    pub(crate) id: String,
    pub(crate) class_list: Rc<RefCell<DOMTokenList>>,
    pub(crate) tag_name: String,
}

impl Element {
    pub fn new(data: ElementData, tag_name: String) -> Self {
        Self {
            data,
            tag_name,
            attributes: HashMap::new(),
            id: String::new(),
            class_list: Rc::new(RefCell::new(DOMTokenList::new())),
        }
    }
}

