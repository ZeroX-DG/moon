#[derive(Debug)]
pub struct CharacterData {
    data: String,
}

impl CharacterData {
    pub fn new(data: String) -> Self {
        Self {
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

