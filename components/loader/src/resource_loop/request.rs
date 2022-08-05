use std::sync::Arc;

use url::Url;
use super::error::LoadError;

pub type Bytes = Vec<u8>;

pub trait FetchListener: Sync + Send {
    fn on_queued(&self) {}
    fn on_started(&self) {}
    #[allow(unused)]
    fn on_finished(&self, bytes: Bytes) {}
    #[allow(unused)]
    fn on_errored(&self, error: LoadError) {}
}

pub struct LoadRequest {
    url: Url,
    listener: Arc<dyn FetchListener>,
}

impl LoadRequest {
    pub fn new(url: Url, listener: Arc<dyn FetchListener>) -> Self {
        Self { url, listener }
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn listener(&self) -> Arc<dyn FetchListener> {
        self.listener.clone()
    }
}
