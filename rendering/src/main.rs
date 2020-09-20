use html::tree_builder::TreeBuilder;
use html::tokenizer::Tokenizer;

fn main() {
    let html = include_str!("../fixtures/test.html");
    let tokenizer = Tokenizer::new(html.to_owned());
    let mut tree_builder = TreeBuilder::new(tokenizer);
    tree_builder.run();
    println!("{:#?}", tree_builder.get_document());
}
