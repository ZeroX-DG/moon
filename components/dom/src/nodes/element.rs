use std::ops::Deref;
use crate::dom_token_list::DOMTokenList;

pub struct Element<T: ElementData> {
    inner: T,
    tag_name: String,
    id: String,
    class_list: DOMTokenList
}

impl<T: ElementData> Element<T> {
    pub fn tag_name(&self) -> String {
        self.tag_name.clone()
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn class_list(&self) -> &DOMTokenList {
        &self.class_list
    }

    pub fn class_name(&self) -> String {
        self.class_list.value()
    }
}

impl<T: ElementData> Deref for Element<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub trait ElementData {}
