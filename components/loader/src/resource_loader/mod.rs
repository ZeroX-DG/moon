pub mod error;
pub mod request;

use flume::{Sender, unbounded, bounded};
use net::http::HttpResponse;
use url::{Url, parser::URLParser};

use self::{request::{LoadRequest, Bytes}, error::LoadError};

pub struct ResourceLoader(Sender<LoadRequest>);

async fn fetch_resource_bytes(url: &Url) -> Result<Bytes, LoadError> {
    match url.scheme.as_str() {
        "file" => std::fs::read(url.path.as_str())
            .map_err(|e| LoadError::IOError(e.to_string())),
        "http" | "https" => match net::http::request("GET", &url.as_str()).await
        {
            HttpResponse::Success(bytes) => Ok(bytes),
            HttpResponse::Failure(err) => Err(LoadError::IOError(err)),
        },
        protocol => Err(LoadError::UnsupportedProtocol(protocol.to_string())),
    }
}

async fn load(url: &Url) -> Result<Bytes, LoadError> {
    match url.scheme.as_str() {
        "view-source" => {
            let target_url = URLParser::parse(&url.path.as_str(), None)
                .ok_or_else(|| LoadError::InvalidURL(url.as_str()))?;
            fetch_resource_bytes(&target_url).await
        }
        _ => fetch_resource_bytes(url).await
    }
}

impl ResourceLoader {
    pub fn new() -> Self {
        let (request_tx, request_rx) = unbounded::<LoadRequest>();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();

            loop {
                let request = request_rx.recv().unwrap();
                let url = request.url();
                let response = rt.block_on(load(url));

                request.response(response);
            }
        });

        Self(request_tx)
    }

    fn load(&mut self, url: &Url) -> Result<Bytes, LoadError> {
        let (tx, rx) = bounded(1);
        let request = LoadRequest::new(url.clone(), tx);

        self.request_sender().send(request)
            .map_err(|_| LoadError::LoaderDisconnected)?;

        rx.recv()
            .map_err(|_| LoadError::LoaderDisconnected)?
    }
}

