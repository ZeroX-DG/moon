use io::input_stream::CharInputStream;

#[derive(Debug, PartialEq, Eq)]
pub enum ParseUrlError {
    InvalidCharacterInProtocol(char),
    InvalidEndOfProtocol,
    UnexpectedEndOfString,
    InvalidCharacterInPort(char),
    InvalidPort(String),
}

#[derive(Debug)]
pub struct Url {
    raw_url: String,
    protocol_end: u32,
    host_start: u32,
    host_end: u32,
    path_start: u32,
    path_end: u32,
    port: Option<u16>,
}

enum ParseState {
    InProtocol,
    InHost,
    InPort,
    InPath,
}

macro_rules! expect_or_throw {
    ($source:expr, $ch:expr, $err:expr) => {
        match $source {
            Some($ch) => {}
            _ => return Err($err),
        }
    };
}

impl Url {
    pub fn parse(input: &str) -> Result<Self, ParseUrlError> {
        let mut stream = CharInputStream::new(input.trim().chars());
        let raw_url = input.to_string();

        let mut state = ParseState::InProtocol;
        let mut index = 0;

        let mut url = Self {
            raw_url,
            protocol_end: 0,
            host_start: 0,
            host_end: 0,
            path_start: 0,
            path_end: 0,
            port: None,
        };
        let mut buffer = String::new();

        loop {
            let next_ch = stream.next();

            if next_ch.is_none() {
                match state {
                    ParseState::InProtocol => return Err(ParseUrlError::UnexpectedEndOfString),
                    ParseState::InPath => url.path_end = index + 1,
                    ParseState::InHost => url.host_end = index,
                    ParseState::InPort => {
                        let port = match buffer.parse::<u16>() {
                            Ok(p) => p,
                            _ => return Err(ParseUrlError::InvalidPort(buffer)),
                        };
                        url.port = Some(port);
                    }
                }

                break;
            }

            let ch = next_ch.unwrap();

            match state {
                ParseState::InProtocol => {
                    match ch {
                        ':' => {
                            url.protocol_end = index - 1;
                            state = ParseState::InHost;

                            expect_or_throw!(
                                stream.next(),
                                '/',
                                ParseUrlError::InvalidEndOfProtocol
                            );
                            expect_or_throw!(
                                stream.next(),
                                '/',
                                ParseUrlError::InvalidEndOfProtocol
                            );

                            // skip useless // at the end of protocol
                            index += 2;
                            url.host_start = index + 1;
                        }
                        'a'..='z' => {
                            index += 1;
                            continue;
                        }
                        c => return Err(ParseUrlError::InvalidCharacterInProtocol(c)),
                    }
                }
                ParseState::InHost => match ch {
                    '/' => {
                        url.host_end = index;
                        url.path_start = index + 1;
                        state = ParseState::InPath;
                    }
                    ':' => {
                        url.host_end = index;
                        state = ParseState::InPort;
                    }
                    _ => {
                        index += 1;
                        continue;
                    }
                },
                ParseState::InPort => match ch {
                    '/' => {
                        let port = match buffer.parse::<u16>() {
                            Ok(p) => p,
                            _ => return Err(ParseUrlError::InvalidPort(buffer)),
                        };
                        url.port = Some(port);
                        buffer.clear();

                        url.path_start = index + 1;
                    }
                    c if c.is_numeric() => {
                        buffer.push(c);
                        index += 1;
                        continue;
                    }
                    c => return Err(ParseUrlError::InvalidCharacterInPort(c)),
                },
                ParseState::InPath => match ch {
                    '?' => {
                        url.path_end = index;
                        break;
                    }
                    _ => {
                        index += 1;
                        continue;
                    }
                },
            }
        }

        Ok(url)
    }

    pub fn host(&self) -> &str {
        &self.raw_url[self.host_start as usize..=self.host_end as usize]
    }

    pub fn protocol(&self) -> &str {
        &self.raw_url[0..=self.protocol_end as usize]
    }

    pub fn path(&self) -> &str {
        if self.path_start == 0 || self.path_end == 0 {
            return "";
        }
        &self.raw_url[self.path_start as usize..=self.path_end as usize]
    }

    pub fn port(&self) -> Option<u16> {
        self.port
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let input_url = "http://google.com/index.php";

        let url = Url::parse(input_url).ok().unwrap();

        assert_eq!(url.protocol(), "http");
        assert_eq!(url.host(), "google.com");
        assert_eq!(url.port(), None);
        assert_eq!(url.path(), "/index.php");
    }

    #[test]
    fn empty_path() {
        let input_url = "http://google.com";

        let url = Url::parse(input_url).ok().unwrap();

        assert_eq!(url.protocol(), "http");
        assert_eq!(url.host(), "google.com");
        assert_eq!(url.port(), None);
        assert_eq!(url.path(), "");
    }

    #[test]
    fn with_port() {
        let input_url = "https://google.com:443";

        let url = Url::parse(input_url).ok().unwrap();

        assert_eq!(url.protocol(), "https");
        assert_eq!(url.host(), "google.com");
        assert_eq!(url.port().unwrap(), 443);
        assert_eq!(url.path(), "");
    }

    #[test]
    fn invalid_protocol() {
        let input_url = "htt1ps://google.com:443";

        let url = Url::parse(input_url);

        assert_eq!(
            url.err().unwrap(),
            ParseUrlError::InvalidCharacterInProtocol('1')
        );
    }

    #[test]
    fn invalid_port() {
        let input_url = "https://google.com:44a3";

        let url = Url::parse(input_url);

        assert_eq!(
            url.err().unwrap(),
            ParseUrlError::InvalidCharacterInPort('a')
        );
    }
}
