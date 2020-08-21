mod insert_mode;

use insert_mode::InsertMode;
use dom::node::NodeRef;
use super::tokenizer::token::Token;

pub struct TreeBuilder {
    open_elements: Vec<NodeRef>,
    insert_mode: InsertMode
}

impl TreeBuilder {
    pub fn new() -> Self {
        Self {
            open_elements: Vec::new(),
            insert_mode: InsertMode::Initial
        }
    }
}
