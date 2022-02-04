use std::rc::Rc;

use css::parser::Parser;
use css::tokenizer::token::Token;
use css::tokenizer::Tokenizer;
use document_loader::inprocess::InprocessLoader;
use dom::document::Document;
use dom::node::{Node, NodeData};

pub struct FrameLoader;

impl FrameLoader {
    pub fn load_html(html: String) -> Rc<Node> {
        let document = Rc::new(Node::new(NodeData::Document(Document::new())));
        document.as_document().set_loader(InprocessLoader::new());

        let default_css = include_str!("../html.css");
        let tokenizer = Tokenizer::new(default_css.chars());
        let mut parser = Parser::<Token>::new(tokenizer.run());
        let stylesheet = parser.parse_a_css_stylesheet();
        document.as_document().append_stylesheet(stylesheet);

        let tokenizer = html::tokenizer::Tokenizer::new(html.chars());
        let tree_builder = html::tree_builder::TreeBuilder::new(tokenizer, document);
        tree_builder.run()
    }
}
