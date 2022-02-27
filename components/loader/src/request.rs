use url::Url;

#[derive(Debug)]
pub enum LoadError {
    UnsupportedProtocol(String),
    IOError(String),
}

type Bytes = Vec<u8>;

pub struct LoadRequest {
    pub url: Url,
    pub success_callback: Option<Box<dyn FnOnce(Bytes)>>,
    pub error_callback: Option<Box<dyn FnOnce(LoadError)>>,
}

impl LoadRequest {
    pub fn new(url: Url) -> Self {
        Self {
            url,
            success_callback: None,
            error_callback: None,
        }
    }

    pub fn on_success<C: FnOnce(Bytes) + 'static>(mut self, callback: C) -> Self {
        self.success_callback = Some(Box::new(callback));
        self
    }

    pub fn on_error<C: FnOnce(LoadError) + 'static>(mut self, callback: C) -> Self {
        self.error_callback = Some(Box::new(callback));
        self
    }
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[LoadError] {:?}", self)
    }
}
