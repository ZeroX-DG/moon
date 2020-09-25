pub mod token;

use std::env;
use io::input_stream::InputStream;
use token::Token;

fn is_trace() -> bool {
    match env::var("TRACE_TOKENIZER") {
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

// TODO: replace with char::REPLACEMENT_CHARACTER when stable
const REPLACEMENT_CHARACTER: char = '\u{FFFD}';

#[allow(non_camel_case_types)]
#[derive(Clone)]
pub enum Char {
    ch(char),
    eof
}

fn is_whitespace(ch: char) -> bool {
    match ch {
        '\t' | '\n' | '\x0C' | ' ' => true,
        _ => false
    }
}

pub struct Tokenizer {
    /// chars input stream for tokenizer
    input: InputStream,

    /// should the tokenizer reconsume the current char
    reconsume_char: bool,

    /// current processing character
    current_character: char
}

impl Tokenizer {
    pub fn new(input: String) -> Self {
        Self {
            input: InputStream::new(input),
            reconsume_char: false,
            current_character: '\0'
        }
    }

    fn consume_next(&mut self) -> Char {
        let ch = if self.reconsume_char {
            // reset reconsume flag
            self.reconsume_char = false;

            Some(self.current_character)
        } else {
            self.input.next()
        };

        match ch {
            Some(c) => {
                self.current_character = c;
                Char::ch(c)
            }
            None => Char::eof,
        }
    }

    fn consume_while<F: Fn(char) -> bool>(&mut self, test: F) {
        while let Some(ch) = self.input.peek_next_char() {
            if !test(ch) {
                return
            }
            self.consume_next();
        }
    }

    fn reconsume(&mut self) {
        self.reconsume_char = true;
    }
}

impl Tokenizer {
    pub fn consume_token(&mut self) -> Token {
        self.consume_comments();

        match self.consume_next() {
            Char::ch(c) if is_whitespace(c) => {
                self.consume_while(is_whitespace);
                Token::Whitespace
            }
            Char::ch('"') => {
                self.consume_string(None)
            }
            _ => {}
        }
    }

    fn consume_comments(&mut self) {
        loop {
            if self.input.peek_next(2) == "/*" {
                loop {
                    match self.consume_next() {
                        Char::eof => {
                            emit_error!("Unexpected EOF while consume_comments");
                            return
                        }
                        _ => {}
                    }
                    if self.input.peek_next(2) == "*/" {
                        return
                    }
                }
            } else {
                return
            }
        }
    }
    fn consume_numeric() {}
    fn consume_ident_like() {}
    fn consume_string(&mut self, ending: Option<char>) -> Token {
        let ending_char = if let Some(c) = ending {
            c
        } else {
            self.current_character
        };
        let token = Token::Str(String::new());
        loop {
            let ch = self.consume_next();
            match ch {
                Char::ch(c) if c == ending_char => {
                    return token;
                }
                Char::eof => {
                    emit_error!("Unexpected EOF");
                    return token;
                }
                Char::ch('\n') => {
                    emit_error!("Unexpected newline");
                    self.reconsume();
                    return Token::BadStr;
                }
                Char::ch('\\') => {
                    let next_char = self.input.peek_next_char();
                    if next_char.is_none() {
                        continue
                    }
                    if let Some('\n') = next_char {
                        self.consume_next();
                        continue
                    }
                    token.append_to_string_token(self.consume_escaped());
                }
                Char::ch(c) => {
                    token.append_to_string_token(c);
                }
            }
        }
    }
    fn consume_url() {}
    fn consume_escaped(&mut self) -> char {

    }
}

