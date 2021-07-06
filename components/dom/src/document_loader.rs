use url::Url;

type Bytes = Vec<u8>;
type SuccessCallback = Box<dyn FnOnce(Bytes)>;
type ErrorCallback = Box<dyn FnOnce(String)>;

pub trait DocumentLoader {
    fn load(&mut self, request: LoadRequest);
}

pub struct LoadRequest {
    pub url: Url,
    pub success_callback: Option<SuccessCallback>,
    pub error_callback: Option<ErrorCallback>,
}

impl LoadRequest {
    pub fn new(url: Url) -> Self {
        Self {
            url,
            success_callback: None,
            error_callback: None,
        }
    }

    pub fn on_success(mut self, callback: SuccessCallback) -> Self {
        self.success_callback = Some(callback);
        self
    }

    pub fn on_error(mut self, callback: ErrorCallback) -> Self {
        self.error_callback = Some(callback);
        self
    }
}
