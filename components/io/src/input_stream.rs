/// The input stream to feed to tokenizer.
#[derive(Debug)]
pub struct InputStream {
    input: String,
    index: usize,
}

impl InputStream {
    pub fn new(input: String) -> Self {
        Self { input, index: 0 }
    }

    /// Consume the next character and return it
    pub fn next(&mut self) -> Option<char> {
        let mut iter = self.input[self.index..].char_indices();
        if let Some((_, cur_char)) = iter.next() {
            let (next_pos, _) = iter.next().unwrap_or((1, ' '));
            self.index += next_pos;
            return Some(cur_char);
        }
        None
    }

    /// Peek the next `n` characters as `&str` to avoid allocation
    pub fn peek_next(&mut self, len: usize) -> Option<&str> {
        let value = self.as_str();
        if let Some((index, _)) = value.char_indices().skip(len).next() {
            return Some(&value[..index]);
        }
        None
    }

    /// Peek the next character
    pub fn peek_next_char(&mut self) -> Option<char> {
        self.as_str().chars().next()
    }

    /// Convert current input from the current index to `&str`
    pub fn as_str(&self) -> &str {
        &self.input[self.index..]
    }
}
