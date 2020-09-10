use dom::dom_ref::NodeRef;

pub struct ListOfActiveFormattingElements {
    entries: Vec<Entry>
}

pub enum Entry {
    Element(NodeRef),
    Marker
}

impl ListOfActiveFormattingElements {
    pub fn new() -> Self {
        Self {
            entries: Vec::new()
        }
    }
    
    pub fn add_marker(&mut self) {
        self.entries.push(Entry::Marker);
    }

    pub fn clear_up_to_last_marker(&mut self) {
        while let Some(entry) = self.entries.pop() {
            match entry {
                Entry::Marker => break,
                _ => continue
            }
        }
    }
}
