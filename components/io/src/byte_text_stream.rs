use std::str::Chars;

use encoding::{all::UTF_8, decode};

pub struct ByteTextStream {
    content: String,
}

impl ByteTextStream {
    pub fn new(bytes: &[u8]) -> Self {
        let decode_result = decode(bytes, encoding::DecoderTrap::Replace, UTF_8);

        match decode_result {
            (Ok(result), _) => Self { content: result },
            _ => {
                log::debug!("Unable to decode text bytes");
                Self {
                    content: String::new(),
                }
            }
        }
    }

    pub fn chars(&self) -> Chars {
        self.content.chars()
    }

    pub fn to_string(&self) -> String {
        self.content.clone()
    }
}
