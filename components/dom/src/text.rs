use super::character_data::CharacterData;

pub struct Text {
    pub character_data: CharacterData,
}

impl core::fmt::Debug for Text {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Text({:?})", self.get_data(),)
    }
}

impl Text {
    pub fn new(data: String) -> Self {
        Self {
            character_data: CharacterData::new(data),
        }
    }

    pub fn get_data(&self) -> String {
        self.character_data.get_data()
    }
}
