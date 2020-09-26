pub mod token;

use std::env;
use io::input_stream::InputStream;
use token::Token;
use token::HashType;

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

fn is_surrogate(n: u32) -> bool {
    match n {
        0xD800..=0xDFFF => true,
        _ => false,
    }
}

fn is_named(ch: char) -> bool {
    if ch == '-' {
        return true
    }
    return ch.is_ascii_digit();
}

fn is_valid_escape(value: String) -> bool {
    if value.len() < 2 {
        return false;
    }
    let chars = value.chars();
    if let Some(c) = chars.next() {
        if c != '\\' {
            return false;
        }
    }
    if let Some(c) = chars.next() {
        if c == '\n' {
            return false;
        }
    }
    return true;
}

fn is_name_start(ch: char) -> bool {
    return ch.is_ascii() || ch >= '\u{0080}' || ch == '_';
}

fn is_start_identifier(value: String) -> bool {
    if value.len() < 3 {
        return false;
    }

    let chars = value.chars();

    match chars.next() {
        Some('-') => {
            let second = chars.next().unwrap();
            let third = chars.next().unwrap();
            if is_name_start(second) || second == '-' {
                return true;
            }
            if is_valid_escape(format!("{}{}", second, third)) {
                return true;
            }
            return false;
        }
        Some(c) if is_name_start(c) => {
            return true;
        }
        Some('\\') => {
            return is_valid_escape(format!("{}{}", '\\', chars.next().unwrap()));
        }
        _ => return false
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
            Char::ch('#') => {
                let is_hash = if let Some(ch) = self.input.peek_next_char() {
                    if is_named(ch) {
                        true
                    } else {
                        false
                    }
                } else {
                    if is_valid_escape(self.input.peek_next(2)) {
                        true
                    } else {
                        false
                    }
                };
                if is_hash {
                    let mut token = Token::new_hash();
                    if is_start_identifier(self.input.peek_next(3)) {
                        token.set_hash_type(HashType::Id);
                    }
                    token.set_hash_value(self.consume_name());
                    return token
                }
                return Token::Delim(self.current_character);
            }
            Char::ch('\'') => {
                self.consume_string(None)
            }
            Char::ch('(') => Token::ParentheseOpen,
            Char::ch(')') => Token::ParentheseClose,
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
    fn consume_name(&mut self) -> String {
        let mut result = String::new();
        loop {
            let ch = self.consume_next();
            if let Char::ch(c) = ch {
                if is_named(c) {
                    result.push(c);
                    continue;
                }
                let next_ch = self.input.peek_next_char();
                if let Some(next_c) = next_ch {
                    if is_valid_escape(format!("{}{}", c, next_c)) {
                        result.push(self.consume_escaped());
                        continue;
                    }
                }
            }
            self.reconsume();
            return result;
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
        let ch = self.consume_next();
        match ch {
            Char::eof => {
                emit_error!("Unexpected EOF");
                REPLACEMENT_CHARACTER
            }
            Char::ch(c) if c.is_ascii_hexdigit() => {
                let mut hex_value: u32 = c.to_digit(16).unwrap();
                for _ in 0..5 {
                    match self.consume_next() {
                        Char::ch(c) => {
                            if c.is_ascii_hexdigit() {
                                hex_value *= 16;
                                hex_value += c.to_digit(16).unwrap();
                                continue
                            }
                            break
                        }
                        Char::eof => {
                            emit_error!("Unexpected EOF");
                            hex_value = 0xFFFD;
                            break
                        }
                    }
                }
                if let Some(c) = self.input.peek_next_char() {
                    if is_whitespace(c) {
                        self.consume_next();
                    }
                }
                if hex_value == 0x00 {
                    hex_value = 0xFFFD;
                }
                if hex_value > 0x10FFFF {
                    hex_value = 0xFFFD;
                }
                if is_surrogate(hex_value) {
                    hex_value = 0xFFFD;
                }
                return std::char::from_u32(hex_value).unwrap_or(REPLACEMENT_CHARACTER);
            }
            Char::ch(c) => c
        }
    }
}

