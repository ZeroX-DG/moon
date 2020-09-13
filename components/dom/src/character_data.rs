use super::node::Node;

pub struct CharacterData {
    pub node: Node,
    data: String,
}

impl CharacterData {
    pub fn new(data: String) -> Self {
        Self {
            node: Node::new(),
            data,
        }
    }

    pub fn get_data(&self) -> String {
        return self.data.clone();
    }

    pub fn append_data(&mut self, data: &str) {
        self.data.push_str(data);
    }
}
