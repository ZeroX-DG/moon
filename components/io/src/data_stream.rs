/// A generic data stream to use in multiple occasions
#[derive(Debug)]
pub struct DataStream<T> {
    data: Vec<T>,
    index: usize,
}

impl<T> DataStream<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self { data, index: 0 }
    }

    pub fn peek(&self) -> Option<&T> {
        self.data.get(self.index)
    }

    pub fn peek_next(&self, len: usize) -> Vec<&T> {
        self.data.iter().skip(self.index).take(len).collect()
    }

    pub fn is_eos(&self) -> bool {
        self.index >= self.data.len()
    }

    pub fn next(&mut self) -> Option<&T> {
        let current = self.data.get(self.index);
        self.index += 1;
        return current;
    }
}
