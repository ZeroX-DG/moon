pub trait CharacterData {
    fn append_data(&mut self, data: String);
    fn insert_data(&mut self, offset: usize, data: String);
    fn delete_data(&mut self, offset: usize, count: usize);
    fn replace_data(&mut self, offset: usize, count: usize, data: String);
}
