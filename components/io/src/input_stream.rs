/// The char input stream
#[derive(Debug)]
pub struct InputStream {
    input: String,
    index: usize,
    reconsume: bool,
    current_char: char,
    current_index: usize,
}

impl InputStream {
    pub fn new(input: String) -> Self {
        Self {
            input,
            index: 0,
            reconsume: false,
            current_char: '\0',
            current_index: 0,
        }
    }

    /// Consume the next character and return it
    pub fn next(&mut self) -> Option<char> {
        if self.reconsume {
            self.reconsume = false;
            return Some(self.current_char);
        }
        let mut iter = self.input[self.index..].char_indices();
        if let Some((_, cur_char)) = iter.next() {
            let (next_pos, _) = iter.next().unwrap_or((1, ' '));
            self.current_index = self.index;
            self.index += next_pos;
            self.current_char = cur_char;
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

    pub fn reconsume(&mut self) {
        self.reconsume = true;
    }

    /// Peek the next character
    pub fn peek_next_char(&mut self) -> Option<char> {
        if self.reconsume {
            return Some(self.current_char);
        }
        self.as_str().chars().next()
    }

    /// Convert current input from the current index to `&str`
    pub fn as_str(&self) -> &str {
        if self.reconsume {
            return &self.input[self.current_index..];
        }
        &self.input[self.index..]
    }
}
