use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use super::ElementHooks;
use super::ElementMethods;
use crate::node::InsertContext;
use crate::node::NodeHooks;
use crate::node::NodePtr;
use loader::resource_loop::request::FetchListener;
use shared::byte_string::ByteString;
use style_types::ContextualStyleSheet;
use url::Url;

use css::parser::Parser;
use css::tokenizer::{token::Token, Tokenizer};
use url::parser::URLParser;

struct StyleLoaderContext {
    stylesheets: Mutex<Vec<Rc<ContextualStyleSheet>>>,
}

impl FetchListener for StyleLoaderContext {
    fn on_finished(&self, bytes: loader::resource_loop::request::Bytes) {
        let css = ByteString::new(&bytes);
        let tokenizer = Tokenizer::new(css.chars());
        let mut parser = Parser::<Token>::new(tokenizer.run());
        let stylesheet = parser.parse_a_css_stylesheet();

        let stylesheet = ContextualStyleSheet::new(
            stylesheet,
            style_types::CascadeOrigin::Author,
            style_types::CSSLocation::External,
        );

        self.stylesheets.lock().unwrap().push(value)
    }

    fn on_errored(&self, error: loader::resource_loop::error::LoadError) {
        log::error!("Unable to load CSS: {}", error);
    }
}

#[derive(Debug)]
pub struct HTMLLinkElement;

impl HTMLLinkElement {
    pub fn empty() -> Self {
        Self
    }

    pub fn load_stylesheet(&self, url: &Url, document: NodePtr) {
        log::info!("Loading stylesheet from: {}", url);

        let loader = document.as_document().loader();
        loader.fetch(url.clone(), Arc::new(StyleLoaderContext));
    }
}

impl ElementHooks for HTMLLinkElement {}

impl NodeHooks for HTMLLinkElement {
    fn on_inserted(&self, context: InsertContext) {
        let document = context.document;
        let element = context.current_node.as_element();
        let attrs = element.attributes();

        let href_str = attrs.borrow().get_str("href");
        let rel_str = attrs.borrow().get_str("rel");

        let href_url = URLParser::parse(&href_str, document.as_document().base());
        match href_url {
            Some(url) => match rel_str.as_str() {
                "stylesheet" => self.load_stylesheet(&url, document),
                _ => {
                    log::warn!("Unsupported link rel value: {}", rel_str);
                }
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
