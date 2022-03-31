use std::cell::RefCell;
use std::rc::Rc;

use super::ElementHooks;
use super::ElementMethods;
use crate::node::InsertContext;
use crate::node::Node;
use crate::node::NodeHooks;
use loader::ResourceLoader;
use shared::byte_string::ByteString;
use style_types::ContextualStyleSheet;
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
        log::info!("Loading stylesheet from: {}", url);

        match ResourceLoader::load(url.clone()) {
            Ok(bytes) => {
                let css = ByteString::new(&bytes);
                let tokenizer = Tokenizer::new(css.chars());
                let mut parser = Parser::<Token>::new(tokenizer.run());
                let stylesheet = parser.parse_a_css_stylesheet();

                let stylesheet = ContextualStyleSheet::new(
                    stylesheet,
                    style_types::CascadeOrigin::Author,
                    style_types::CSSLocation::External,
                );

                document.as_document().append_stylesheet(stylesheet);
            }
            Err(e) => log::error!("Unable to load CSS: {} ({})", e, url),
        }
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
    fn on_inserted(&self, context: InsertContext) {
        let document = context.document;
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
