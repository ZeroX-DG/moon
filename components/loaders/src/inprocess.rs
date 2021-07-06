use dom::document_loader::{DocumentLoader, LoadRequest};
use relative_path::RelativePath;

pub struct InprocessLoader {}

impl InprocessLoader {
    pub fn new() -> Self {
        Self {}
    }
}

impl DocumentLoader for InprocessLoader {
    fn load(&mut self, request: LoadRequest) {
        match request.url.protocol() {
            "file" => match std::fs::read(request.url.path()) {
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
            },
            "relative" => {
                let path = RelativePath::new(request.url.path())
                    .to_logical_path(std::env::current_dir().unwrap());

                match std::fs::read(path) {
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
            _ => {}
        }
    }
}
