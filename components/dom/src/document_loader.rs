use url::Url;

type Bytes = Vec<u8>;

pub trait DocumentLoader {
    fn load(&mut self, request: LoadRequest);
}

pub struct LoadRequest {
    pub url: Url,
    pub success_callback: Option<Box<dyn FnOnce(Bytes)>>,
    pub error_callback: Option<Box<dyn FnOnce(String)>>,
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

    pub fn on_error<C: FnOnce(String) + 'static>(mut self, callback: C) -> Self {
        self.error_callback = Some(Box::new(callback));
        self
    }
}
