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

#[derive(Debug)]
pub struct HTMLLinkElement {
    href: RefCell<Option<Url>>,
    relationship: RefCell<Option<HTMLLinkRelationship>>,
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
        }
    }

    pub fn load_stylesheet(&self, url: &Url, document: Rc<Node>) {
        let cloned_doc = document.clone();
        let raw_url = url.raw().to_string();

        log::info!("Loading stylesheet from: {}", raw_url);

        let request = LoadRequest::new(url.clone())
            .on_success(move |bytes| {
                let css = String::from_utf8(bytes).unwrap();
                let tokenizer = Tokenizer::new(css.chars());
                let mut parser = Parser::<Token>::new(tokenizer.run());
                let stylesheet = parser.parse_a_css_stylesheet();

                cloned_doc.as_document().append_stylesheet(stylesheet);
            })
            .on_error(move |e| log::error!("Unable to load CSS: {} ({})", e, raw_url));

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
                *self.href.borrow_mut() = match Url::parse(value) {
                    Ok(url) => Some(url),
                    Err(_) => {
                        log::info!("Invalid href URL: {}", value);
                        None
                    }
                }
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
        match &*self.href.borrow() {
            Some(url) => match *self.relationship.borrow() {
                Some(HTMLLinkRelationship::Stylesheet) => self.load_stylesheet(url, document),
                _ => {}
            },
            None => log::info!("No URL found, ignoring"),
        }
    }
}

impl ElementMethods for HTMLLinkElement {
    fn tag_name(&self) -> String {
        "link".to_string()
    }
}
