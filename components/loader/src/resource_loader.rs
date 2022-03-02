use net::http::HttpResponse;
use tokio::runtime::Runtime;
use url::Url;

#[derive(Debug)]
pub enum LoadError {
    UnsupportedProtocol(String),
    IOError(String),
}

type Bytes = Vec<u8>;

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[LoadError] {:?}", self)
    }
}

pub struct ResourceLoader;

impl ResourceLoader {
    pub fn load(url: Url) -> Result<Bytes, LoadError> {
        let rt = Runtime::new().unwrap();
        let load_result = match url.scheme.as_str() {
            "file" => {
                std::fs::read(url.path.as_str()).map_err(|e| LoadError::IOError(e.to_string()))
            }
            "http" | "https" => {
                match rt.block_on(net::http::start_http_request("get", &url.as_str())) {
                    HttpResponse::Success(bytes) => Ok(bytes),
                    HttpResponse::Failure(err) => Err(LoadError::IOError(err)),
                }
            }
            protocol => Err(LoadError::UnsupportedProtocol(protocol.to_string())),
        };
        load_result
    }
}
