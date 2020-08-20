mod insert_mode;

use dom::dom_traits::Dom;
use insert_mode::InsertMode;

pub struct TreeBuilder<T: Dom> {
    open_elements: Vec<T>,
    insert_mode: InsertMode
}

impl<T: Dom> TreeBuilder<T> {
    pub fn new() -> Self {
        Self {
            open_elements: Vec::new(),
            insert_mode: InsertMode::Initial
        }
    }
}
