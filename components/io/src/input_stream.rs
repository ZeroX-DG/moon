#[derive(Debug)]
pub struct InputStream {
    input: String,
    index: usize,
}

impl InputStream {
    pub fn new(input: String) -> Self {
        Self { input, index: 0 }
    }

    pub fn next(&mut self) -> Option<char> {
        let mut iter = self.input[self.index..].char_indices();
        if let Some((_, cur_char)) = iter.next() {
            let (next_pos, _) = iter.next().unwrap_or((1, ' '));
            self.index += next_pos;
            return Some(cur_char);
        }
        None
    }

    pub fn peek_next(&mut self, len: usize) -> String {
        self.as_str().chars().take(len).collect()
    }

    pub fn peek_next_char(&mut self) -> Option<char> {
        self.as_str().chars().next()
    }

    pub fn as_str(&self) -> &str {
        &self.input[self.index..]
    }
}
