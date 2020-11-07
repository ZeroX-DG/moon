use dom::dom_ref::NodeRef;

pub fn print_dom_tree(root: NodeRef, level: usize) {
    let child_nodes = root.borrow().as_node().child_nodes();
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
