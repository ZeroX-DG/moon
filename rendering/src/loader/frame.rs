use dom::dom_ref::NodeRef;

pub struct FrameLoader;

impl FrameLoader {
    pub fn load_html(html: String) -> NodeRef {
        let tokenizer = html::tokenizer::Tokenizer::new(html.chars());
        let tree_builder = html::tree_builder::TreeBuilder::new(tokenizer);
        tree_builder.run()
    }
}
