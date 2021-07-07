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

        let tokenizer = html::tokenizer::Tokenizer::new(html.chars());
        let tree_builder = html::tree_builder::TreeBuilder::new(tokenizer, document);
        tree_builder.run()
    }
}
