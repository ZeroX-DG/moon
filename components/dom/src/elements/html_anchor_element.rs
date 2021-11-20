use std::cell::RefCell;

use super::ElementHooks;
use super::ElementMethods;
use crate::node::NodeHooks;
use url::Url;

#[derive(Debug)]
pub struct HTMLAnchorElement {
    href: RefCell<Option<Url>>,
}

impl HTMLAnchorElement {
    pub fn empty() -> Self {
        Self {
            href: RefCell::new(None),
        }
    }
}

impl ElementHooks for HTMLAnchorElement {
    fn on_attribute_change(&self, attr: &str, value: &str) {
        if attr == "href" {
            *self.href.borrow_mut() = Url::parse(value).ok();
        }
    }
}

impl NodeHooks for HTMLAnchorElement {}

impl ElementMethods for HTMLAnchorElement {
    fn tag_name(&self) -> String {
        "a".to_string()
    }
}
