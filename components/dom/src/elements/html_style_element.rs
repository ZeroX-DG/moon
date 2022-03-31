use std::cell::RefCell;
use std::rc::Rc;

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
    stylesheet: RefCell<Option<Rc<ContextualStyleSheet>>>,
}

impl HTMLStyleElement {
    pub fn empty() -> Self {
        Self {
            stylesheet: RefCell::new(None),
        }
    }
}

impl ElementHooks for HTMLStyleElement {}

impl NodeHooks for HTMLStyleElement {
    fn on_children_updated(&self, context: ChildrenUpdateContext) {
        let document = context.document.as_document();
        let css = context.current_node.descendant_text_content();
        let tokenizer = Tokenizer::new(css.chars());
        let mut parser = Parser::<Token>::new(tokenizer.run());
        let stylesheet = parser.parse_a_css_stylesheet();

        let stylesheet = ContextualStyleSheet::new(
            stylesheet,
            style_types::CascadeOrigin::Author,
            style_types::CSSLocation::Embedded,
        );

        if let Some(sheet) = &*self.stylesheet.borrow() {
            document.remove_stylesheet(sheet);
        }

        let stylesheet_ptr = document.append_stylesheet(stylesheet);
        self.stylesheet.replace(Some(stylesheet_ptr));
    }
}

impl ElementMethods for HTMLStyleElement {
    fn tag_name(&self) -> String {
        "style".to_string()
    }
}
