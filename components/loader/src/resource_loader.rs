use net::http::HttpResponse;
use tokio::runtime::Runtime;
use url::{parser::URLParser, Url};

#[derive(Debug)]
pub enum LoadError {
    UnsupportedProtocol(String),
    IOError(String),
    InvalidURL(String),
}

type Bytes = Vec<u8>;

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[LoadError] {:?}", self)
    }
}

impl LoadError {
    pub fn get_friendly_message(&self) -> String {
        match self {
            LoadError::IOError(error) => format!("Unable to load resource from local: {}", error),
            LoadError::UnsupportedProtocol(error) => format!(
                "Unable to load resource from unsupported protocol: {}",
                error
            ),
            LoadError::InvalidURL(error) => {
                format!("Unable to load resource from invalid URL: {}", error)
            }
        }
    }
}

pub struct ResourceLoader;

impl ResourceLoader {
    pub fn load(url: &Url) -> Result<Bytes, LoadError> {
        let rt = Runtime::new().unwrap();
        let load_result = match url.scheme.as_str() {
            "file" => {
                std::fs::read(url.path.as_str()).map_err(|e| LoadError::IOError(e.to_string()))
            }
            "http" | "https" => match rt.block_on(net::http::request("GET", &url.as_str())) {
                HttpResponse::Success(bytes) => Ok(bytes),
                HttpResponse::Failure(err) => Err(LoadError::IOError(err)),
            },
            "view-source" => {
                let target_url = URLParser::parse(&url.path.as_str(), None)
                    .ok_or_else(|| LoadError::InvalidURL(url.as_str()))?;
                ResourceLoader::load(&target_url)
            }
            protocol => Err(LoadError::UnsupportedProtocol(protocol.to_string())),
        };
        load_result
    }
}
