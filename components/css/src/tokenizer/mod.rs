pub mod token;

use std::env;
use io::output_stream::OutputStream;
use io::input_stream::InputStream;
use token::Token;
use token::HashType;
use token::NumberType;
use regex::Regex;

fn is_trace() -> bool {
    match env::var("TRACE_CSS_TOKENIZER") {
        Ok(s) => s == "true" || s == "",
        _ => false,
    }
}

macro_rules! trace {
    ($err:expr) => {
        println!("[ParseError][CSS Tokenization]: {}", $err);
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

/// Check if a code point is a whitespace (according to specs)
/// https://www.w3.org/TR/css-syntax-3/#whitespace
fn is_whitespace(ch: char) -> bool {
    match ch {
        '\t' | '\n' | '\x0C' | ' ' => true,
        _ => false
    }
}

/// Check if a codepoint is non-printable
/// https://www.w3.org/TR/css-syntax-3/#non-printable-code-point
fn is_non_printable(ch: char) -> bool {
    match ch {
        '\u{0000}'..='\u{0008}' => true,
        '\u{000B}' => true,
        '\u{000E}'..='\u{001F}' => true,
        '\u{007F}' => true,
        _ => false
    }
}

/// Check if a codepoint value is a surrogate
fn is_surrogate(n: u32) -> bool {
    match n {
        0xD800..=0xDFFF => true,
        _ => false,
    }
}

/// Check if a codepoint is a name code point
/// https://www.w3.org/TR/css-syntax-3/#name-code-point
fn is_named(ch: char) -> bool {
    return is_name_start(ch) || ch.is_ascii_digit() || ch == '-';
}

/// Check if 2 codepoints are valid escape
/// https://www.w3.org/TR/css-syntax-3/#starts-with-a-valid-escape
fn is_valid_escape(value: &str) -> bool {
    let mut chars = value.chars();
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

/// Check if a codepoint is a name-start codepoint
/// https://www.w3.org/TR/css-syntax-3/#name-start-code-point
fn is_name_start(ch: char) -> bool {
    return ch.is_ascii_alphabetic() || ch >= '\u{0080}' || ch == '_';
}

/// Check if 3 codepoints would start an identifier
/// https://www.w3.org/TR/css-syntax-3/#would-start-an-identifier
fn is_start_identifier(value: &str) -> bool {
    let mut chars = value.chars();

    match chars.next() {
        Some('-') => {
            let second = chars.next().unwrap();
            let third = chars.next().unwrap();
            if is_name_start(second) || second == '-' {
                return true;
            }
            if is_valid_escape(&format!("{}{}", second, third)) {
                return true;
            }
            return false;
        }
        Some(c) if is_name_start(c) => {
            return true;
        }
        Some('\\') => {
            return is_valid_escape(&format!("{}{}", '\\', chars.next().unwrap()));
        }
        _ => return false
    }
}

/// Check if 3 codepoints would start a number
/// https://www.w3.org/TR/css-syntax-3/#starts-with-a-number
fn is_start_number(value: &str) -> bool {
    let mut chars = value.chars();
    let first = chars.next().unwrap();
    let second = chars.next().unwrap();
    let third = chars.next().unwrap();
    
    match first {
        '+' | '-' => {
            if second.is_ascii_digit() || (second == '.' && third.is_ascii_digit()) {
                return true;
            }
            return false;
        }
        '.' => {
            if second.is_ascii_digit() {
                return true;
            }
            return false;
        }
        c if c.is_ascii_digit() => return true,
        _ => return false
    }
}

/// Tokenizer for the CSS stylesheet
pub struct Tokenizer {
    /// chars input stream for tokenizer
    input: InputStream,

    /// current processing character
    current_character: char,

    /// Output tokens
    output: Vec<Token>
}

impl Tokenizer {
    pub fn new(input: String) -> Self {
        Self {
            input: InputStream::new(input),
            current_character: '\0',
            output: Vec::new()
        }
    }

    /// Constantly running the tokenizer and produce a list of tokens
    pub fn run(mut self) -> OutputStream<Token> {
        loop {
            let token = self.consume_token();
            self.output.push(token.clone());

            match token {
                Token::EOF => return OutputStream::new(self.output),
                _ => {}
            }
        }
    }

    fn consume_next(&mut self) -> Char {
        let ch = self.input.next();

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
        self.input.reconsume();
    }
}

impl Tokenizer {
    /// Consume and return the next token
    /// Should only be use for testing, use `run()` when you want to run tokenizer
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
                    if let Some(next_2_chars) = self.input.peek_next(2) {
                        is_valid_escape(next_2_chars)
                    } else {
                        false
                    }
                };
                if is_hash {
                    let mut token = Token::new_hash();
                    if let Some(next_3_chars) = self.input.peek_next(3) {
                        if is_start_identifier(next_3_chars) {
                            token.set_hash_type(HashType::Id);
                        }
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
            Char::ch('+') => {
                if let Some(next_2_chars) = self.input.peek_next(2) {
                    if is_start_number(&format!("+{}", next_2_chars)) {
                        self.reconsume();
                        return self.consume_numeric();
                    }
                }
                return Token::Delim(self.current_character);
            }
            Char::ch(',') => Token::Comma,
            Char::ch('-') => {
                if let Some(next_2_chars) = self.input.peek_next(2) {
                    if is_start_number(&format!("-{}", next_2_chars)) {
                        self.reconsume();
                        return self.consume_numeric();
                    }
                    if next_2_chars == "->" {
                        self.consume_next();
                        self.consume_next();
                        return Token::CDC;
                    }
                    if is_start_identifier(&format!("-{}", next_2_chars)) {
                        self.reconsume();
                        return self.consume_ident_like();
                    }
                }
                return Token::Delim(self.current_character);
            }
            Char::ch('.') => {
                if let Some(next_2_chars) = self.input.peek_next(2) {
                    if is_start_number(&format!(".{}", next_2_chars)) {
                        self.reconsume();
                        return self.consume_numeric();
                    }
                }
                return Token::Delim(self.current_character);
            }
            Char::ch(':') => Token::Colon,
            Char::ch(';') => Token::Semicolon,
            Char::ch('<') => {
                if let Some("!--") = self.input.peek_next(3) {
                    self.consume_next();
                    self.consume_next();
                    self.consume_next();
                    return Token::CDO;
                }
                return Token::Delim(self.current_character);
            }
            Char::ch('@') => {
                if let Some(next_3_chars) = self.input.peek_next(3) {
                    if is_start_identifier(next_3_chars) {
                        return Token::AtKeyword(self.consume_name());
                    }
                }
                return Token::Delim(self.current_character);
            }
            Char::ch('[') => Token::BracketOpen,
            Char::ch('\\') => {
                if let Some(ch) = self.input.peek_next_char() {
                    if is_valid_escape(&format!("\\{}", ch)) {
                        self.reconsume();
                        return self.consume_ident_like();
                    }
                }
                emit_error!("Unexpected escape sequence");
                return Token::Delim(self.current_character);
            }
            Char::ch(']') => Token::BracketClose,
            Char::ch('{') => Token::BraceOpen,
            Char::ch('}') => Token::BraceClose,
            Char::ch(c) if c.is_ascii_digit() => {
                self.reconsume();
                return self.consume_numeric();
            }
            Char::ch(c) if is_name_start(c) => {
                self.reconsume();
                return self.consume_ident_like();
            }
            Char::eof => Token::EOF,
            _ => Token::Delim(self.current_character)
        }
    }

    fn consume_comments(&mut self) {
        'outer: loop {
            if let Some(next_2_chars) = self.input.peek_next(2) {
                if next_2_chars != "/*" {
                    break
                }
                self.consume_next(); // /
                self.consume_next(); // *
                loop {
                    if let Some(end_comment) = self.input.peek_next(2) {
                        if end_comment == "*/" {
                            self.consume_next();
                            self.consume_next();
                            break
                        } else {
                            self.consume_next();
                        }
                    } else {
                        emit_error!("Unexpected EOF while consume_comments");
                        break 'outer
                    }
                }
                continue;
            }
            break;
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
                    if is_valid_escape(&format!("{}{}", c, next_c)) {
                        result.push(self.consume_escaped());
                        continue;
                    }
                }
            }
            self.reconsume();
            return result;
        }
    }

    fn consume_numeric(&mut self) -> Token {
        let (number, type_) = self.consume_number();
        if let Some(next_3_chars) = self.input.peek_next(3) {
            if is_start_identifier(next_3_chars) {
                return Token::Dimension {
                    value: number,
                    type_,
                    unit: self.consume_name()
                };
            }
        }
        if let Some('%') = self.input.peek_next_char() {
            self.consume_next();
            return Token::Percentage(number);
        }
        return Token::Number { value: number, type_ };
    }

    fn consume_number(&mut self) -> (i32, NumberType) {
        fn consume_while_number_and_append_to_repr(this: &mut Tokenizer, repr: &mut String) {
            loop {
                if let Some(c) = this.input.peek_next_char() {
                    if c.is_ascii_digit() {
                        this.consume_next();
                        repr.push(c);
                        continue
                    }
                }
                break
            }
        }
        let mut type_ = NumberType::Integer;
        let mut repr  = String::new();
        if let Some(c) = self.input.peek_next_char() {
            if c == '+' || c == '-' {
                self.consume_next();
                repr.push(c);
            }
        }
        consume_while_number_and_append_to_repr(self, &mut repr);
        if let Some(next_2_chars) = self.input.peek_next(2) {
            let mut chars = next_2_chars.chars();
            let first = chars.next().unwrap();
            let last = chars.next().unwrap();
            if first == '.' && last.is_ascii_digit() {
                self.consume_next();
                self.consume_next();
                repr.push(first);
                repr.push(last);
                type_ = NumberType::Number;

                consume_while_number_and_append_to_repr(self, &mut repr);
            }
        }
        if let Some(next_3_chars) = self.input.peek_next(3) {
            let re = Regex::new(r"^(e|E)(\+|-)?\d$").unwrap();
            if let Some(match_len) = re.shortest_match(next_3_chars) {
                for _ in 0..match_len {
                    if let Char::ch(c) = self.consume_next() {
                        repr.push(c);
                    }
                }
                type_ = NumberType::Number;
                consume_while_number_and_append_to_repr(self, &mut repr);
            }
        }
        let value = i32::from_str_radix(&repr, 10).unwrap();
        return (value, type_);
    }
    
    fn consume_ident_like(&mut self) -> Token {
        let string = self.consume_name();
        if string.eq_ignore_ascii_case("url") {
            if let Some('(') = self.input.peek_next_char() {
                self.consume_next();
                loop {
                    if let Some(next_2_chars) = self.input.peek_next(2) {
                        let mut chars = next_2_chars.chars();
                        let first = chars.next().unwrap();
                        let second = chars.next().unwrap();
                        if is_whitespace(first) && is_whitespace(second) {
                            self.consume_next();
                        } else {
                            break
                        }
                    }
                }
                if let Some(next_2_chars) = self.input.peek_next(2) {
                    let re = Regex::new("^ ?('|\")$").unwrap();
                    if re.is_match(next_2_chars) {
                        return Token::Function(string);
                    }
                    return self.consume_url();
                }
            }
        }
        if let Some('(') = self.input.peek_next_char() {
            return Token::Function(string);
        }
        return Token::Ident(string);
    }

    fn consume_string(&mut self, ending: Option<char>) -> Token {
        let ending_char = if let Some(c) = ending {
            c
        } else {
            self.current_character
        };
        let mut token = Token::Str(String::new());
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

    fn consume_url(&mut self) -> Token {
        let mut token = Token::Url(String::new());
        self.consume_while(is_whitespace);
        loop {
            match self.consume_next() {
                Char::ch(')') => return token,
                Char::eof => {
                    emit_error!("Unexpected EOF");
                    return token;
                }
                Char::ch(c) if is_whitespace(c) => {
                    self.consume_while(is_whitespace);
                    if let Some(c) = self.input.peek_next_char() {
                        if c == ')' {
                            return token;
                        }
                    } else {
                        emit_error!("Unexpected EOF");
                        return token;
                    }
                    self.consume_bad_url();
                    return Token::BadUrl;
                }
                Char::ch('"') | Char::ch('\'') | Char::ch('(') => {
                    emit_error!("Unexpected character");
                    self.consume_bad_url();
                    return Token::BadUrl;
                }
                Char::ch(c) if is_non_printable(c) => {
                    emit_error!("Unexpected non-printable character");
                    self.consume_bad_url();
                    return Token::BadUrl;
                }
                Char::ch('\\') => {
                    if let Some(c) = self.input.peek_next_char() {
                        if is_valid_escape(&format!("\\{}", c)) {
                            token.append_to_url_token(self.consume_escaped());
                        } else {
                            emit_error!("Unexpected escape sequence");
                            self.consume_bad_url();
                            return Token::BadUrl;
                        }
                    }
                }
                _ => {
                    token.append_to_url_token(self.current_character);
                }
            }
        }
    }

    fn consume_bad_url(&mut self) {
        loop {
            match self.consume_next() {
                Char::ch(')') | Char::eof => return,
                _ => {}
            }
            if let Some(ch) = self.input.peek_next_char() {
                if is_valid_escape(&format!("{}{}", self.current_character, ch)) {
                    self.consume_escaped();
                }
            }
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_simple_css() {
        let css = r"#id_selector .class_selector {
            color: red;
            background: url(https://example.com/image.jpg);
        }".to_string();
        let mut tokenizer = Tokenizer::new(css);
        assert_eq!(tokenizer.consume_token(), Token::Hash("id_selector".to_string(), HashType::Id));
        assert_eq!(tokenizer.consume_token(), Token::Whitespace);
        
        assert_eq!(tokenizer.consume_token(), Token::Delim('.'));
        assert_eq!(tokenizer.consume_token(), Token::Ident("class_selector".to_string()));
        assert_eq!(tokenizer.consume_token(), Token::Whitespace);

        assert_eq!(tokenizer.consume_token(), Token::BraceOpen);

        assert_eq!(tokenizer.consume_token(), Token::Whitespace);

        assert_eq!(tokenizer.consume_token(), Token::Ident("color".to_string()));
        assert_eq!(tokenizer.consume_token(), Token::Colon);
        assert_eq!(tokenizer.consume_token(), Token::Whitespace);
        assert_eq!(tokenizer.consume_token(), Token::Ident("red".to_string()));
        assert_eq!(tokenizer.consume_token(), Token::Semicolon);

        assert_eq!(tokenizer.consume_token(), Token::Whitespace);

        assert_eq!(tokenizer.consume_token(), Token::Ident("background".to_string()));
        assert_eq!(tokenizer.consume_token(), Token::Colon);
        assert_eq!(tokenizer.consume_token(), Token::Whitespace);
        assert_eq!(tokenizer.consume_token(), Token::Url("https://example.com/image.jpg".to_string()));
        assert_eq!(tokenizer.consume_token(), Token::Semicolon);

        assert_eq!(tokenizer.consume_token(), Token::Whitespace);
        
        assert_eq!(tokenizer.consume_token(), Token::BraceClose);
        assert_eq!(tokenizer.consume_token(), Token::EOF);
    }
}
