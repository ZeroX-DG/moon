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

    pub fn peek(&self, len: usize) -> Option<String> {
        if len == 0 {
            return None
        }
        let chars = self.input[self.index..].chars();
        return Some(chars.take(len).collect());
    }

    pub fn as_str(&self) -> String {
        self.input[self.index..].to_string()
    }
}
