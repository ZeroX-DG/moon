use std::path::Path;

use url::{parser::URLParser, Url};

pub struct Browser {
    home_url: Url,
}

impl Browser {
    pub fn new() -> Self {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_str()
            .unwrap();
        Self {
            home_url: URLParser::parse(
                &format!("file://{}/fixtures/test.html", workspace_root),
                None,
            )
            .unwrap(),
        }
    }

    pub fn home_url(&self) -> &Url {
        &self.home_url
    }
}
