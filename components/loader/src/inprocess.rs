use super::{DocumentLoader, LoadError, LoadRequest};

pub struct InprocessLoader;

impl InprocessLoader {
    pub fn new() -> Self {
        Self
    }
}

impl DocumentLoader for InprocessLoader {
    fn load(&mut self, request: LoadRequest) {
        let load_result = match request.url.scheme.as_str() {
            "file" => std::fs::read(request.url.path.as_str())
                .map_err(|e| LoadError::IOError(e.to_string())),
            protocol => Err(LoadError::UnsupportedProtocol(protocol.to_string())),
        };

        match load_result {
            Ok(bytes) => {
                if let Some(cb) = request.success_callback {
                    cb(bytes);
                }
            }
            Err(e) => {
                if let Some(cb) = request.error_callback {
                    cb(e);
                }
            }
        }
    }
}
