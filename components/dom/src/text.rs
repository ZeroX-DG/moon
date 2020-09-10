use super::character_data::CharacterData;

pub struct Text {
    pub character_data: CharacterData
}

impl Text {
    pub fn new(data: String) -> Self {
        Self {
            character_data: CharacterData::new(data)
        }
    }

    pub fn get_data(&self) -> String {
        self.character_data.get_data()
    }
}
