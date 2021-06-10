use relative_path::RelativePath;
use url::Url;

type Bytes = Vec<u8>;

pub struct DocumentLoader {}

pub struct LoadRequest<'a, T, S, E, M>
where
    S: FnOnce(T),
    E: FnOnce(String),
    M: FnOnce(Bytes) -> T,
{
    url: &'a Url,
    success_callback: Option<S>,
    error_callback: Option<E>,
    map_fn: M,
}

impl DocumentLoader {
    pub fn new() -> Self {
        Self {}
    }

    pub fn load<T, S, E, M>(&mut self, request: LoadRequest<T, S, E, M>)
    where
        S: FnOnce(T),
        E: FnOnce(String),
        M: FnOnce(Bytes) -> T,
    {
        match request.url.protocol() {
            "file" => match std::fs::read(request.url.path()) {
                Ok(bytes) => {
                    if let Some(cb) = request.success_callback {
                        let data = (request.map_fn)(bytes);
                        cb(data);
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
                            let data = (request.map_fn)(bytes);
                            cb(data);
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

impl<'a, T, S, E, M> LoadRequest<'a, T, S, E, M>
where
    S: FnOnce(T),
    E: FnOnce(String),
    M: FnOnce(Bytes) -> T,
{
    pub fn new(url: &'a Url, map_fn: M) -> Self {
        Self {
            url,
            success_callback: None,
            error_callback: None,
            map_fn,
        }
    }

    pub fn on_success(mut self, callback: S) -> Self {
        self.success_callback = Some(callback);
        self
    }

    pub fn on_error(mut self, callback: E) -> Self {
        self.error_callback = Some(callback);
        self
    }
}
