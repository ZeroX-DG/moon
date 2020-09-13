use super::character_data::CharacterData;

pub struct Comment {
    pub character_data: CharacterData,
}

impl Comment {
    pub fn new(data: String) -> Self {
        Self {
            character_data: CharacterData::new(data),
        }
    }

    pub fn get_data(&self) -> String {
        self.character_data.get_data()
    }
}
