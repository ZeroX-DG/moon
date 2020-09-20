use html::tree_builder::TreeBuilder;
use html::tokenizer::Tokenizer;
use dom::dom_ref::NodeRef;

fn print_tree(root: NodeRef, level: usize) {
    if level == 0 {
        println!(" {:#?}", root);
    } else {
        println!("{}└{:#?}", " ".repeat(level), root);
    }
    for node in root.borrow().as_node().child_nodes() {
        print_tree(node, level + 1);
    }
}

fn main() {
    let html = include_str!("../fixtures/test.html");
    let tokenizer = Tokenizer::new(html.to_owned());
    let mut tree_builder = TreeBuilder::new(tokenizer);
    tree_builder.run();

    let document = tree_builder.get_document();
    println!("-----------------------");
    print_tree(document, 0);
}
