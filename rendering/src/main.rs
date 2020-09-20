use dom::dom_ref::NodeRef;
use html::tokenizer::Tokenizer;
use html::tree_builder::TreeBuilder;

fn print_tree(root: NodeRef, level: usize) {
    let child_nodes = root.borrow().as_node().child_nodes();
    println!(
        "{}{:#?}({} child)",
        "    ".repeat(level),
        root,
        child_nodes.length()
    );
    for node in child_nodes {
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
