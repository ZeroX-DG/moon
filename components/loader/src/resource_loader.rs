use flume::{unbounded, Sender};
use net::http::HttpResponse;
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

pub struct LoadRequest {
    url: Url,
    response_tx: Sender<Result<Bytes, LoadError>>,
}
tokio::task_local! {
    pub static RESOURCE_LOADER: ResourceLoader;
}

lazy_static::lazy_static! {
    static ref GLOBAL_RESOURCE_LOADER: ResourceLoader = ResourceLoader::init();
}

#[derive(Clone)]
pub struct ResourceLoader(Sender<LoadRequest>);

impl ResourceLoader {
    pub fn init() -> Self {
        let (request_tx, request_rx) = unbounded();

        let loader = ResourceLoader(request_tx);
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();

            fn load(url: &Url, rt: &tokio::runtime::Runtime) -> Result<Vec<u8>, LoadError> {
                let load_result = match url.scheme.as_str() {
                    "file" => std::fs::read(url.path.as_str())
                        .map_err(|e| LoadError::IOError(e.to_string())),
                    "http" | "https" => match rt.block_on(net::http::request("GET", &url.as_str()))
                    {
                        HttpResponse::Success(bytes) => Ok(bytes),
                        HttpResponse::Failure(err) => Err(LoadError::IOError(err)),
                    },
                    "view-source" => {
                        let target_url = URLParser::parse(&url.path.as_str(), None)
                            .ok_or_else(|| LoadError::InvalidURL(url.as_str()))?;
                        load(&target_url, rt)
                    }
                    protocol => Err(LoadError::UnsupportedProtocol(protocol.to_string())),
                };
                load_result
            }

            loop {
                let request = request_rx.recv().unwrap();
                let url = request.url;
                let response = load(&url, &rt);

                request.response_tx.send(response).unwrap();
            }
        });

        loader
    }

    pub fn current() -> Self {
        RESOURCE_LOADER.with(|loader| loader.clone())
    }

    pub fn global() -> &'static Self {
        &GLOBAL_RESOURCE_LOADER
    }

    pub fn load(&self, url: &Url) -> Result<Bytes, LoadError> {
        let (tx, rx) = flume::bounded(1);
        self.0
            .send(LoadRequest {
                url: url.clone(),
                response_tx: tx,
            })
            .unwrap();
        rx.recv().unwrap()
    }
}
