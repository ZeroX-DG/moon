use dom::document_loader::{DocumentLoader, LoadRequest};
use relative_path::RelativePath;

use crate::error::LoadError;

pub struct InprocessLoader;

impl InprocessLoader {
    pub fn new() -> Self {
        Self
    }
}

impl DocumentLoader for InprocessLoader {
    fn load(&mut self, request: LoadRequest) {
        let load_result = match request.url.protocol() {
            "file" => {
                std::fs::read(request.url.path()).map_err(|e| LoadError::IOError(e.to_string()))
            }
            "relative" => {
                let path = RelativePath::new(request.url.path())
                    .to_logical_path(std::env::current_dir().unwrap());
                std::fs::read(path).map_err(|e| LoadError::IOError(e.to_string()))
            }
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
                    cb(e.to_string());
                }
            }
        }
    }
}
