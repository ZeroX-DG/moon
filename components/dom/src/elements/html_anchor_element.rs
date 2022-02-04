use std::cell::RefCell;
use std::rc::Rc;

use super::ElementHooks;
use super::ElementMethods;
use crate::node::Node;
use crate::node::NodeHooks;
use url::parser::URLParser;
use url::Url;

#[derive(Debug)]
pub struct HTMLAnchorElement {
    href: RefCell<Option<Url>>,
    _raw_href: RefCell<String>,
}

impl HTMLAnchorElement {
    pub fn empty() -> Self {
        Self {
            href: RefCell::new(None),
            _raw_href: RefCell::new(String::new()),
        }
    }
}

impl ElementHooks for HTMLAnchorElement {
    fn on_attribute_change(&self, attr: &str, value: &str) {
        if attr == "href" {
            *self._raw_href.borrow_mut() = value.to_string();
        }
    }
}

impl NodeHooks for HTMLAnchorElement {
    fn on_inserted(&self, document: Rc<Node>) {
        let base = document.as_document().base();
        *self.href.borrow_mut() = URLParser::parse(&self._raw_href.borrow(), base);
    }
}

impl ElementMethods for HTMLAnchorElement {
    fn tag_name(&self) -> String {
        "a".to_string()
    }
}
