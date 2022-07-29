use flume::Sender;
use url::Url;

use super::error::LoadError;

pub type Bytes = Vec<u8>;

pub struct LoadRequest {
    url: Url,
    response_tx: Sender<Result<Bytes, LoadError>>,
}

impl LoadRequest {

    pub fn new(url: Url, response_tx: Sender<Result<Bytes, LoadError>>) -> Self {
        Self {
            url,
            response_tx
        }
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn response(&self, response: Result<Bytes, LoadError>) {
        self.response_tx.send(response).expect("Unable to send response for request");
    }
}

