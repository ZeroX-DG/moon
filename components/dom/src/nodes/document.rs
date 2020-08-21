use crate::node::{NodeRef, NodeData};

pub struct Document<'a> {
    doctype: Option<NodeRef<'a>>
}

impl<'a> Document<'a> {
    pub fn new() -> Self {
        Self {
            doctype: None
        }
    }
}

impl<'a> NodeData for Document<'a> {}
