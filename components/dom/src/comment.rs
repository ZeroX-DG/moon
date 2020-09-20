use super::character_data::CharacterData;

pub struct Comment {
    pub character_data: CharacterData,
}

impl core::fmt::Debug for Comment {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Comment({:?}) at {:#?}", self.get_data(), self as *const Comment)
    }
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
