use css::parser::Parser;
use css::tokenizer::token::Token;
use css::tokenizer::Tokenizer;
use dom::document::Document;
use dom::dom_ref::NodeRef;
use dom::node::{Node, NodeData};
use loaders::inprocess::InprocessLoader;

pub struct FrameLoader;

impl FrameLoader {
    pub fn load_html(html: String) -> NodeRef {
        let document = NodeRef::new(Node::new(NodeData::Document(Document::new())));
        document
            .borrow_mut()
            .as_document_mut()
            .set_loader(InprocessLoader::new());

        let default_css = include_str!("../html.css");
        let tokenizer = Tokenizer::new(default_css.chars());
        let mut parser = Parser::<Token>::new(tokenizer.run());
        let stylesheet = parser.parse_a_css_stylesheet();
        document
            .borrow_mut()
            .as_document_mut()
            .append_stylesheet(stylesheet);

        let tokenizer = html::tokenizer::Tokenizer::new(html.chars());
        let tree_builder = html::tree_builder::TreeBuilder::new(tokenizer, document);
        tree_builder.run()
    }
}
