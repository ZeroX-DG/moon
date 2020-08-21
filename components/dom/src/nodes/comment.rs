use crate::node::NodeData;

pub struct Comment(String);

impl Comment {
    pub fn new(data: String) -> Self {
        Self(data)
    }
}

impl NodeData for Comment {}
