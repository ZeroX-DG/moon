use std::cell::RefCell;
use std::rc::Rc;
use std::ops::Deref;
use super::dom_token_list::DOMTokenList;
use super::node::NodeData;

pub struct Element<T: ElementData> {
    inner: Rc<RefCell<T>>,
    tag_name: String,
    id: String,
    class_list: DOMTokenList
}

impl<T: ElementData> Element<T> {
    pub fn class_name(&self) -> String {
        self.class_list.value()
    }
}

impl<T: ElementData> Deref for Element<T> {
    type Target = Rc<RefCell<T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}


pub trait ElementData {}

impl<T: ElementData> NodeData for Element<T> {}
