#[derive(Debug)]
pub struct HTMLAnchorElement {
}

impl HTMLAnchorElement {
    pub fn href(&self) -> String {
        self.node_ref.borrow().as_element().attributes().get_str("href")
    }

    pub fn text(&self) -> String {
        let mut result = String::new();
        for child in self.node_ref.borrow().child_nodes() {
            result.push_str(&child.borrow().descendant_text_content());
        }
        result
    }
}

