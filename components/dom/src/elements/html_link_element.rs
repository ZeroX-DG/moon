use std::sync::Arc;
use std::sync::Mutex;

use super::ElementHooks;
use super::ElementMethods;
use crate::node::InsertContext;
use crate::node::NodeHooks;
use crate::node::NodePtr;
use flume::bounded;
use flume::Sender;
use loader::resource_loop::request::FetchListener;
use shared::byte_string::ByteString;
use style_types::ContextualStyleSheet;
use url::Url;

use css::parser::Parser;
use css::tokenizer::{token::Token, Tokenizer};
use url::parser::URLParser;

struct StyleLoaderContext {
    stylesheet_tx: Sender<ContextualStyleSheet>,
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

        self.stylesheet_tx.send(stylesheet).unwrap();
    }

    fn on_errored(&self, error: loader::resource_loop::error::LoadError) {
        log::error!("Unable to load CSS: {}", error);
    }
}

#[derive(Debug)]
pub struct HTMLLinkElement {
    stylesheet: Arc<Mutex<Option<ContextualStyleSheet>>>,
}

impl HTMLLinkElement {
    pub fn empty() -> Self {
        Self {
            stylesheet: Arc::new(Mutex::new(None)),
        }
    }

    pub fn load_stylesheet(&self, url: &Url, document: NodePtr) {
        log::info!("Loading stylesheet from: {}", url);

        let stylesheet = self.stylesheet.clone();
        let (tx, rx) = bounded(1);

        let loader = document.as_document().loader();
        loader.fetch(url.clone(), StyleLoaderContext { stylesheet_tx: tx });

        // This is blocking the main thread manually. In the future, this receiving should run on a separate thread
        // and the main thread should wait for that thread to finish, while working on other things.
        match rx.recv() {
            Ok(sheet) => {
                stylesheet.lock().unwrap().replace(sheet);
            }
            _ => {}
        }
    }

    pub fn stylesheet(&self) -> Arc<Mutex<Option<ContextualStyleSheet>>> {
        self.stylesheet.clone()
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
                "stylesheet" => {
                    document
                        .as_document()
                        .register_style_element(context.current_node);
                    self.load_stylesheet(&url, document);
                }
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
