use std::cell::RefCell;
use std::rc::Rc;

use super::ElementHooks;
use super::ElementMethods;
use crate::node::Node;
use crate::node::NodeHooks;
use document_loader::LoadRequest;
use url::Url;

use css::parser::Parser;
use css::tokenizer::{token::Token, Tokenizer};
use url::parser::URLParser;

#[derive(Debug)]
pub struct HTMLLinkElement {
    href: RefCell<Option<Url>>,
    relationship: RefCell<Option<HTMLLinkRelationship>>,
    _raw_href: RefCell<String>,
}

#[derive(Debug)]
pub enum HTMLLinkRelationship {
    Stylesheet,
}

impl HTMLLinkElement {
    pub fn empty() -> Self {
        Self {
            href: RefCell::new(None),
            relationship: RefCell::new(None),
            _raw_href: RefCell::new(String::new()),
        }
    }

    pub fn load_stylesheet(&self, url: &Url, document: Rc<Node>) {
        let cloned_doc = document.clone();
        let cloned_url = url.clone();

        log::info!("Loading stylesheet from: {}", url);

        let request = LoadRequest::new(url.clone())
            .on_success(move |bytes| {
                let css = String::from_utf8(bytes).unwrap();
                let tokenizer = Tokenizer::new(css.chars());
                let mut parser = Parser::<Token>::new(tokenizer.run());
                let stylesheet = parser.parse_a_css_stylesheet();

                cloned_doc.as_document().append_stylesheet(stylesheet);
            })
            .on_error(move |e| log::error!("Unable to load CSS: {} ({})", e, cloned_url));

        let loader = document
            .as_document()
            .loader()
            .expect("Document loader is not set");
        loader.borrow_mut().load(request);
    }
}

impl ElementHooks for HTMLLinkElement {
    fn on_attribute_change(&self, attr: &str, value: &str) {
        match attr {
            "href" => {
                *self._raw_href.borrow_mut() = value.to_string();
            }
            "rel" => {
                if value == "stylesheet" {
                    *self.relationship.borrow_mut() = Some(HTMLLinkRelationship::Stylesheet);
                }
            }
            _ => {}
        }
    }
}

impl NodeHooks for HTMLLinkElement {
    fn on_inserted(&self, document: Rc<Node>) {
        let href_url = &*self._raw_href.borrow();
        *self.href.borrow_mut() = URLParser::parse(href_url, document.as_document().base());
        match &*self.href.borrow() {
            Some(url) => match *self.relationship.borrow() {
                Some(HTMLLinkRelationship::Stylesheet) => self.load_stylesheet(url, document),
                _ => {}
            },
            None => log::info!("Empty or invalid URL, ignoring"),
        }
    }
}

impl ElementMethods for HTMLLinkElement {
    fn tag_name(&self) -> String {
        "link".to_string()
    }
}
