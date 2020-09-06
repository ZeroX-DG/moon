#[derive(Debug)]
pub struct DOMTokenList {
    items: Vec<String>
}

impl DOMTokenList {
    pub fn new() -> Self {
        Self {
            items: Vec::new()
        }
    }

    pub fn length(&self) -> usize {
        self.items.len()
    }

    pub fn item(&self, index: usize) -> Option<String> {
        match self.items.get(index) {
            Some(item) => Some(item.clone()),
            _ => None
        }
    }

    pub fn contains(&self, token: &str) -> bool {
        self.items.contains(&token.to_owned())
    }

    pub fn add(&mut self, tokens: Vec<String>) {
        let mut tokens = tokens;
        self.items.append(&mut tokens);
    }

    pub fn remove(&mut self, tokens: Vec<String>) {
        self.items.retain(|item| !tokens.contains(item));
    }

    pub fn value(&self) -> String {
        self.items.join(" ")
    }
}
