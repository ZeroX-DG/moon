mod insert_mode;

use insert_mode::InsertMode;
use dom::node::NodeRef;
use super::tokenizer::token::Token;

pub struct TreeBuilder {
    open_elements: Vec<NodeRef>,
    insert_mode: InsertMode
}

pub enum TreeBuildingStatus {
    Continue,
    Stop
}

impl TreeBuilder {
    pub fn new() -> Self {
        Self {
            open_elements: Vec::new(),
            insert_mode: InsertMode::Initial
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
