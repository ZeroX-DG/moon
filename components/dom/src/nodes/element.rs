use crate::dom_token_list::DOMTokenList;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

pub enum ElementData {
    AnchorElement
}

pub struct Element {
    pub(crate) data: ElementData,
    pub(crate) attributes: HashMap<String, String>,
    pub(crate) id: String,
    pub(crate) class_list: Rc<RefCell<DOMTokenList>>
}

impl Element {
    pub fn new(data: ElementData) -> Self {
        Self {
            data,
            attributes: HashMap::new(),
            id: String::new(),
            class_list: Rc::new(RefCell::new(DOMTokenList::new()))
        }
    }
}

