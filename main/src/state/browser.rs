use url::{Url, parser::URLParser};

pub struct Browser {
    home_url: Url
}

impl Browser {
    pub fn new() -> Self {
        Self {
            home_url: URLParser::parse("file:///home/zerox/Desktop/Projects/moon/fixtures/test.html", None).unwrap()
        }
    }

    pub fn home_url(&self) -> &Url {
        &self.home_url
    }
}

