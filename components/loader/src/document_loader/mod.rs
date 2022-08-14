use std::sync::Arc;
use flume::Sender;
use url::Url;

use crate::resource_loop::{request::{LoadRequest, FetchListener}};

#[derive(Clone)]
pub struct DocumentLoader {
    resource_loop_tx: Sender<LoadRequest>,
}

impl DocumentLoader {
    pub fn new(resource_loop_tx: Sender<LoadRequest>) -> Self {
        Self {
            resource_loop_tx
        }
    }

    pub fn fetch(&self, url: Url, listener: impl FetchListener + 'static) {
        let request = LoadRequest::new(url, Arc::new(listener));
        self.resource_loop_tx.send(request).expect("Unable to send fetch request");
    }
}
