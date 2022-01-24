#[derive(Debug)]
pub enum LoadError {
    UnsupportedProtocol(String),
    IOError(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[LoadError] {:?}", self)
    }
}
