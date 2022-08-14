use std::cell::Ref;
use std::cell::RefCell;

use css::parser::Parser;
use css::tokenizer::token::Token;
use css::tokenizer::Tokenizer;
use style_types::ContextualStyleSheet;

use super::ElementHooks;
use super::ElementMethods;
use crate::node::ChildrenUpdateContext;
use crate::node::NodeHooks;

#[derive(Debug)]
pub struct HTMLStyleElement {
    stylesheet: RefCell<Option<ContextualStyleSheet>>,
}

impl HTMLStyleElement {
    pub fn empty() -> Self {
        Self {
            stylesheet: RefCell::new(None),
        }
    }

    pub fn stylesheet(&self) -> Ref<Option<ContextualStyleSheet>> {
        self.stylesheet.borrow()
    }
}

impl ElementHooks for HTMLStyleElement {}

impl NodeHooks for HTMLStyleElement {
    fn on_inserted(&self, context:crate::node::InsertContext) {
        context.document.as_document().register_style_element(context.current_node);
    }

    fn on_children_updated(&self, context: ChildrenUpdateContext) {
        let css = context.current_node.descendant_text_content();
        let tokenizer = Tokenizer::new(css.chars());
        let mut parser = Parser::<Token>::new(tokenizer.run());
        let stylesheet = parser.parse_a_css_stylesheet();

        let stylesheet = ContextualStyleSheet::new(
            stylesheet,
            style_types::CascadeOrigin::Author,
            style_types::CSSLocation::Embedded,
        );

        self.stylesheet.replace(Some(stylesheet));
    }
}

impl ElementMethods for HTMLStyleElement {
    fn tag_name(&self) -> String {
        "style".to_string()
    }
}
