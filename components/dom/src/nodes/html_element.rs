pub enum HTMLElementData {
}

pub struct HTMLElement {
    data: HTMLElementData
}

impl HTMLElement {
    pub fn new(data: HTMLElementData) -> Self {
        Self {
            data
        }
    }
}

