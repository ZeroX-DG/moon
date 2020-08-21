pub struct DocumentType {
    pub name: String,
    pub public_id: String,
    pub system_id: String
}

impl DocumentType {
    pub fn new(name: String, public_id: String, system_id: String) -> Self {
        Self {
            name,
            public_id,
            system_id
        }
    }
}

