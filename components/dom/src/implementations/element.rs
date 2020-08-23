use crate::node::{NodeRef, NodeInner};
use crate::dom_token_list::DOMTokenList;
use std::rc::Rc;
use std::cell::RefCell;
use super::Node;

pub trait Element : Node {
    fn id(&self) -> String;
    fn class_name(&self) -> String;
    fn class_list(&mut self) -> Rc<RefCell<DOMTokenList>>;
    fn get_attribute(&self, attr: &str) -> Option<String>;
    fn set_attribute(&mut self, attr: &str, value: &str);
}

const PANIC_NON_ELEMENT: &'static str = "Node is not an element";

impl Element for NodeRef {
    fn id(&self) -> String {
        let ref_self = self.borrow();
        if let NodeInner::Element(element) = &*ref_self.inner.borrow() {
            return element.id.clone()
        }
        panic!(PANIC_NON_ELEMENT)
    }

    fn class_name(&self) -> String {
        let ref_self = self.borrow();
        if let NodeInner::Element(element) = &*ref_self.inner.borrow() {
            return element.class_list.borrow().value()
        }
        panic!(PANIC_NON_ELEMENT)
    }

    fn class_list(&mut self) -> Rc<RefCell<DOMTokenList>> {
        let ref_self = self.borrow();
        if let NodeInner::Element(element) = &mut *ref_self.inner.borrow_mut() {
            return element.class_list.clone()
        }
        panic!(PANIC_NON_ELEMENT) 
    }

    fn get_attribute(&self, attr: &str) -> Option<String> {
        let ref_self = self.borrow();
        if let NodeInner::Element(element) = &*ref_self.inner.borrow() {
            return match element.attributes.get(attr) {
                Some(val) => Some(val.clone()),
                None => None
            }
        }
        panic!(PANIC_NON_ELEMENT)
    }

    fn set_attribute(&mut self, attr: &str, value: &str) {
        let ref_self = self.borrow();
        if let NodeInner::Element(element) = &mut *ref_self.inner.borrow_mut() {
            element.attributes.insert(attr.to_owned(), value.to_owned());
            return
        }
        panic!(PANIC_NON_ELEMENT)
    }
}
