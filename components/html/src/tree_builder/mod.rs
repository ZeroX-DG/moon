use dom::dom_traits::Dom;

pub struct TreeBuilder<T: Dom> {
    open_elements: Vec<T>
}

impl<T: Dom> TreeBuilder<T> {
    pub fn new() -> Self {
        Self {
            open_elements: Vec::new()
        }
    }
}
