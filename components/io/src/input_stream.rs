/// The char input stream
#[derive(Debug)]
pub struct InputStream {
    input: String,
    index: usize,
    consumed_index: usize,
    reconsume: bool,
    consumed: Option<char>,
    is_last_ch: bool,
}

impl InputStream {
    pub fn new(input: String) -> Self {
        Self {
            input,
            index: 0,
            consumed_index: 0,
            reconsume: false,
            consumed: None,
            is_last_ch: false,
        }
    }

    /// Consume the current character and return it
    pub fn next(&mut self) -> Option<char> {
        if self.reconsume {
            self.reconsume = false;
            return self.consumed;
        }
        if self.is_last_ch {
            return None;
        }
        let mut indexes = self.input[self.index..].char_indices();
        if let Some((_, consumed_char)) = indexes.next() {
            self.consumed = Some(consumed_char);
            self.consumed_index = self.index;
            if let Some((offset, _)) = indexes.next() {
                self.index += offset;
            } else {
                self.is_last_ch = true;
            }
            return self.consumed;
        }
        return None;
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
        if self.is_last_ch {
            return None;
        }
        self.as_str().chars().next()
    }

    /// Convert current input from the current index to `&str`
    pub fn as_str(&self) -> &str {
        if self.reconsume {
            return &self.input[self.consumed_index..];
        }
        if self.is_last_ch {
            return "";
        }
        &self.input[self.index..]
    }
}
