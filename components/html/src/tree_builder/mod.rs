mod insert_mode;

use insert_mode::InsertMode;
use dom::node::{NodeType, NodeRef};
use dom::nodes::document::Document;
use super::tokenizer::token::Token;

pub struct TreeBuilder {
    // stack of open elements as mentioned in specs
    open_elements: Vec<NodeRef>,

    // current insertion mode
    insert_mode: InsertMode,

    // the result document
    document: NodeRef
}

pub enum TreeBuildingStatus {
    Continue,
    Stop
}

impl TreeBuilder {
    pub fn new() -> Self {
        Self {
            open_elements: Vec::new(),
            insert_mode: InsertMode::Initial,
            document: NodeRef::new_node(NodeType::Document, Document::new())
        }
    }

    pub fn feed(&mut self, token: Token) -> TreeBuildingStatus {
        match self.insert_mode {
            InsertMode::Initial => self.handle_initial(token),
            _ => unimplemented!()
        }
    }

    fn handle_initial(&mut self, token: Token) -> TreeBuildingStatus {
        TreeBuildingStatus::Continue
    }
}
