pub mod token;

use io::input_stream::InputStream;
use std::env;
use token::Token;

pub struct Tokenizer {
    // chars input stream for tokenizer
    input: InputStream,
}

pub enum Char {
    ch(char),
    EOF
}

fn is_whitespace(c: char) -> bool {
    match c {
        '\t' | '\n' | ' ' => true,
        _ => false,
    }
}

fn is_trace() -> bool {
    match env::var("TRACE_CSS_TOKENIZER") {
        Ok(s) => s == "true",
        _ => false,
    }
}

macro_rules! trace {
    ($err:expr) => {
        println!("[ParseError][Tokenization]: {}", $err);
    };
}

macro_rules! emit_error {
    ($err:expr) => {
        if is_trace() {
            trace!($err)
        }
    };
}

impl Tokenizer {
    pub fn new(input: String) -> Self {
        Self {
            input: InputStream::new(input),
        }
    }

    pub fn consume_comment(&mut self) {
        loop {
            if let Some(text) = self.input.peek(2) {
                if text == "/*" {
                    if !self.consume_until_inclusive("*/") {
                        emit_error!("Unexpected EOF while consuming comment");
                        return
                    }
                } else {
                    return
                }
            }
        }
    }

    fn consume_whitespaces(&mut self) {
        loop {
            if let Char::ch(c) = self.next_ch() {
                if c != ' ' {
                    return
                }
            }
        }
    }

    fn consume_until_inclusive(&mut self, pattern: &str) -> bool {
        loop {
            let chars = pattern.chars();
            let mut finished = true;
            for c in chars {
                match self.next_ch() {
                    Char::ch(ch) => {
                        if c != ch {
                            finished = false;
                            break
                        }
                    }
                    Char::EOF => {
                        return false;
                    }
                }
            }
            if finished {
                return true;
            }
        }
    }

    fn next_ch(&mut self) -> Char {
        if let Some(ch) = self.input.next() {
            return Char::ch(ch);
        }
        Char::EOF
    }
}
