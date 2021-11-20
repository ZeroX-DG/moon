use std::cell::RefCell;

#[derive(Debug)]
pub struct CharacterData {
    data: RefCell<String>,
}

impl CharacterData {
    pub fn new(data: String) -> Self {
        Self { data: RefCell::new(data) }
    }

    pub fn get_data(&self) -> String {
        return self.data.borrow().clone();
    }

    pub fn append_data(&self, data: &str) {
        self.data.borrow_mut().push_str(data);
    }
}
