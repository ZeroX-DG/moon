#[derive(Debug)]
pub enum LoadError {
    UnsupportedProtocol(String),
    IOError(String),
    InvalidURL(String),
    LoaderDisconnected
}

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
            LoadError::LoaderDisconnected => {
                format!("ResourceLoader disconnected unexpectedly.")
            }
        }
    }
}

