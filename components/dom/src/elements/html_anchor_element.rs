use std::cell::RefCell;

use super::ElementHooks;
use super::ElementMethods;
use crate::node::InsertContext;
use crate::node::NodeHooks;
use url::parser::URLParser;
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

impl ElementHooks for HTMLAnchorElement {}

impl NodeHooks for HTMLAnchorElement {
    fn on_inserted(&self, context: InsertContext) {
        let document = context.document;
        let base = document.as_document().base();

        let element = context.current_node.as_element();
        let href_str = element.attributes().borrow().get_str("href");
        *self.href.borrow_mut() = URLParser::parse(&href_str, base);
    }
}

impl ElementMethods for HTMLAnchorElement {
    fn tag_name(&self) -> String {
        "a".to_string()
    }
}
