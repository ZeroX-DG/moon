use dom::node::NodePtr;

pub fn print_dom_tree(root: NodePtr, level: usize) {
    let child_nodes = root.child_nodes();
    println!(
        "{}{:#?}({} child)",
        "    ".repeat(level),
        root,
        child_nodes.length()
    );
    for node in child_nodes {
        print_dom_tree(NodePtr(node), level + 1);
    }
}
