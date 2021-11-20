use std::rc::Rc;

use dom::node::Node;

pub fn print_dom_tree(root: Rc<Node>, level: usize) {
    let child_nodes = root.child_nodes();
    println!(
        "{}{:#?}({} child)",
        "    ".repeat(level),
        root,
        child_nodes.length()
    );
    for node in child_nodes {
        print_dom_tree(node, level + 1);
    }
}
