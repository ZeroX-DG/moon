/// The output stream to receive token tokenizer.
#[derive(Debug)]
pub struct OutputStream<T> {
    output: Vec<T>,
    index: usize,
}

impl<T> OutputStream<T> {
    pub fn new(output: Vec<T>) -> Self {
        Self { output, index: 0 }
    }

    pub fn peek(&self) -> Option<&T> {
        self.output.get(self.index + 1)
    }

    pub fn next(&mut self) -> Option<&T> {
        self.index += 1;
        self.output.get(self.index)
    }
}
