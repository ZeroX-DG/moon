pub mod state;
pub mod token;

use super::entities::ENTITIES;
use io::input_stream::InputStream;
use state::State;
use std::collections::{HashSet, VecDeque};
use std::env;
use token::Attribute;
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

fn is_surrogate(n: u32) -> bool {
    match n {
        0xD800..=0xDFFF => true,
        _ => false,
    }
}

fn is_nonecharacter(n: u32) -> bool {
    match n {
        0xfdd0..=0xfdef
        | 0xfffe..=0xffff
        | 0x1_fffe..=0x1_ffff
        | 0x2_fffe..=0x2_ffff
        | 0x3_fffe..=0x3_ffff
        | 0x4_fffe..=0x4_ffff
        | 0x5_fffe..=0x5_ffff
        | 0x6_fffe..=0x6_ffff
        | 0x7_fffe..=0x7_ffff
        | 0x8_fffe..=0x8_ffff
        | 0x9_fffe..=0x9_ffff
        | 0xA_fffe..=0xA_ffff
        | 0xB_fffe..=0xB_ffff
        | 0xC_fffe..=0xC_ffff
        | 0xD_fffe..=0xD_ffff
        | 0xE_fffe..=0xE_ffff
        | 0xF_fffe..=0xF_ffff
        | 0x10_fffe..=0x10_ffff => true,
        _ => false,
    }
}

fn is_control(n: u32) -> bool {
    match n {
        0x0000..=0x001F => true,
        0x007F..=0x009F => true,
        _ => false,
    }
}

fn is_whitespace(n: u32) -> bool {
    match n {
        0x0009 | 0x000A | 0x000C | 0x000D | 0x0020 => true,
        _ => false,
    }
}

pub fn replace_control_codes(n: u32) -> Option<u32> {
    match n {
        0x80 => Some(0x20AC),
        0x82 => Some(0x201A),
        0x83 => Some(0x0192),
        0x84 => Some(0x201E),
        0x85 => Some(0x2026),
        0x86 => Some(0x2020),
        0x87 => Some(0x2021),
        0x88 => Some(0x02C6),
        0x89 => Some(0x2030),
        0x8A => Some(0x0160),
        0x8B => Some(0x2039),
        0x8C => Some(0x0152),
        0x8E => Some(0x017D),
        0x91 => Some(0x2018),
        0x92 => Some(0x2019),
        0x93 => Some(0x201C),
        0x94 => Some(0x201D),
        0x95 => Some(0x2022),
        0x96 => Some(0x2013),
        0x97 => Some(0x2014),
        0x98 => Some(0x02DC),
        0x99 => Some(0x2122),
        0x9A => Some(0x0161),
        0x9B => Some(0x203A),
        0x9C => Some(0x0153),
        0x9E => Some(0x017E),
        0x9F => Some(0x0178),
        _ => None,
    }
}

// TODO: replace with char::REPLACEMENT_CHARACTER when stable
const REPLACEMENT_CHARACTER: char = '\u{FFFD}';

#[allow(non_camel_case_types)]
#[derive(Clone)]
pub enum Char {
    ch(char),
    eof,
    null,
    whitespace,
}

pub struct Tokenizer {
    // chars input stream for tokenizer
    input: InputStream,

    // A list of tokenized tokens
    output: VecDeque<Token>,

    // Current consumed character. Might reconsume later
    current_character: char,

    // The state for tokenizing
    state: State,

    // The return state
    return_state: Option<State>,

    // Current token
    current_token: Option<Token>,

    // Specify if the next step should reconsume the current char
    reconsume_char: bool,

    // Temporary buffer to track progress
    temp_buffer: String,

    // Last emitted start tag to verify if end tag is valid
    last_emitted_start_tag: Option<Token>,

    // Code for a character reference. Example: &#228;
    character_reference_code: u32,
}

impl Tokenizer {
    pub fn new(input: String) -> Self {
        Self {
            input: InputStream::new(input),
            output: VecDeque::new(),
            current_character: '\0',
            state: State::Data,
            return_state: None,
            current_token: None,
            reconsume_char: false,
            temp_buffer: String::new(),
            last_emitted_start_tag: None,
            character_reference_code: 0,
        }
    }

    pub fn next_token(&mut self) -> Token {
        if !self.output.is_empty() {
            return self.output.pop_front().unwrap();
        }
        loop {
            match self.state {
                State::Data => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('&') => {
                            self.return_state = Some(State::Data);
                            self.switch_to(State::CharacterReference);
                        }
                        Char::ch('<') => self.switch_to(State::TagOpen),
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            return self.emit_current_char();
                        }
                        Char::eof => return self.emit_eof(),
                        _ => return self.emit_current_char(),
                    }
                }
                State::RCDATA => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('&') => {
                            self.return_state = Some(State::RCDATA);
                            self.switch_to(State::CharacterReference);
                        }
                        Char::ch('<') => self.switch_to(State::RCDATALessThanSign),
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            return self.emit_char(REPLACEMENT_CHARACTER);
                        }
                        Char::eof => return self.emit_eof(),
                        _ => return self.emit_current_char(),
                    }
                }
                State::RAWTEXT => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('<') => self.switch_to(State::RAWTEXTLessThanSign),
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            return self.emit_char(REPLACEMENT_CHARACTER);
                        }
                        Char::eof => return self.emit_eof(),
                        _ => return self.emit_current_char(),
                    }
                }
                State::ScriptData => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('<') => self.switch_to(State::ScriptDataLessThanSign),
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            self.emit_char(REPLACEMENT_CHARACTER);
                        }
                        Char::eof => return self.emit_eof(),
                        _ => return self.emit_current_char(),
                    }
                }
                State::PLAINTEXT => {
                    let ch = self.consume_next();
                    match ch {
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            return self.emit_char(REPLACEMENT_CHARACTER);
                        }
                        Char::eof => return self.emit_eof(),
                        _ => return self.emit_current_char(),
                    }
                }
                State::TagOpen => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('!') => self.switch_to(State::MarkupDeclarationOpen),
                        Char::ch('/') => self.switch_to(State::EndTagOpen),
                        Char::ch(c) if c.is_ascii_alphabetic() => {
                            self.current_token = Some(Token::new_start_tag());
                            self.reconsume_in(State::TagName);
                        }
                        Char::ch('?') => {
                            emit_error!("unexpected-question-mark-instead-of-tag-name");
                            self.new_token(Token::new_comment(""));
                            self.reconsume_in(State::BogusComment);
                        }
                        Char::eof => {
                            emit_error!("eof-before-tag-name");
                            self.will_emit(Token::Character('<'));
                            return self.emit_eof();
                        }
                        _ => {
                            emit_error!("invalid-first-character-of-tag-name");
                            self.will_emit(Token::Character('<'));
                            self.reconsume_in(State::Data);
                        }
                    }
                }
                State::EndTagOpen => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch(c) if c.is_ascii_alphabetic() => {
                            self.new_token(Token::new_end_tag());
                            self.reconsume_in(State::TagName);
                        }
                        Char::ch('>') => {
                            emit_error!("missing-end-tag-name");
                            self.switch_to(State::Data);
                        }
                        Char::eof => {
                            emit_error!("eof-before-tag-name");
                            self.will_emit(Token::Character('<'));
                            self.will_emit(Token::Character('/'));
                            return self.emit_eof();
                        }
                        _ => {
                            emit_error!("invalid-first-character-of-tag-name");
                            self.new_token(Token::new_comment(""));
                            self.reconsume_in(State::BogusComment);
                        }
                    }
                }
                State::TagName => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace => {
                            self.switch_to(State::BeforeAttributeName);
                        }
                        Char::ch('/') => {
                            self.switch_to(State::SelfClosingStartTag);
                        }
                        Char::ch('>') => {
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::ch(c) if c.is_ascii_uppercase() => {
                            self.append_character_to_tag_name(c.to_ascii_lowercase());
                        }
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            self.append_character_to_tag_name(REPLACEMENT_CHARACTER);
                        }
                        Char::eof => {
                            emit_error!("eof-in-tag");
                            return self.emit_eof();
                        }
                        _ => {
                            self.append_character_to_tag_name(self.current_character);
                        }
                    }
                }
                State::RCDATALessThanSign => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('/') => {
                            self.temp_buffer.clear();
                            self.switch_to(State::RCDATAEndTagOpen);
                        }
                        _ => {
                            self.will_emit(Token::Character('<'));
                            self.reconsume_in(State::RCDATA);
                        }
                    }
                }
                State::RCDATAEndTagOpen => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch(c) if c.is_ascii_alphabetic() => {
                            self.new_token(Token::new_end_tag());
                            self.reconsume_in(State::RCDATAEndTagName);
                        }
                        _ => {
                            self.will_emit(Token::Character('<'));
                            self.will_emit(Token::Character('/'));
                            self.reconsume_in(State::RCDATA);
                        }
                    }
                }
                State::RCDATAEndTagName => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace => {
                            if !self.is_end_tag_appropriate() {
                                self.will_emit(Token::Character('<'));
                                self.will_emit(Token::Character('/'));
                                self.emit_temp_buffer();
                                self.reconsume_in(State::RCDATA);
                            } else {
                                self.switch_to(State::BeforeAttributeName);
                            }
                        }
                        Char::ch('/') => {
                            if !self.is_end_tag_appropriate() {
                                self.will_emit(Token::Character('<'));
                                self.will_emit(Token::Character('/'));
                                self.emit_temp_buffer();
                                self.reconsume_in(State::RCDATA);
                            } else {
                                self.switch_to(State::SelfClosingStartTag);
                            }
                        }
                        Char::ch('>') => {
                            if !self.is_end_tag_appropriate() {
                                self.will_emit(Token::Character('<'));
                                self.will_emit(Token::Character('/'));
                                self.emit_temp_buffer();
                                self.reconsume_in(State::RCDATA);
                            } else {
                                self.switch_to(State::Data);
                                return self.emit_current_token();
                            }
                        }
                        Char::ch(c) if c.is_ascii_uppercase() => {
                            self.append_character_to_tag_name(c.to_ascii_lowercase());
                            self.temp_buffer.push(self.current_character);
                        }
                        Char::ch(c) if c.is_ascii_lowercase() => {
                            self.append_character_to_tag_name(self.current_character);
                            self.temp_buffer.push(self.current_character);
                        }
                        _ => {
                            self.will_emit(Token::Character('<'));
                            self.will_emit(Token::Character('/'));
                            self.emit_temp_buffer();
                            self.reconsume_in(State::RCDATA);
                        }
                    }
                }
                State::RAWTEXTLessThanSign => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('/') => {
                            self.temp_buffer.clear();
                            self.switch_to(State::RAWTEXTEndTagOpen);
                        }
                        _ => {
                            self.will_emit(Token::Character('<'));
                            self.reconsume_in(State::RAWTEXT);
                        }
                    }
                }
                State::RAWTEXTEndTagOpen => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch(c) if c.is_ascii_alphabetic() => {
                            self.new_token(Token::new_end_tag());
                            self.reconsume_in(State::RAWTEXTEndTagName);
                        }
                        _ => {
                            self.will_emit(Token::Character('<'));
                            self.will_emit(Token::Character('/'));
                            self.reconsume_in(State::RAWTEXT);
                        }
                    }
                }
                State::RAWTEXTEndTagName => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace => {
                            if !self.is_end_tag_appropriate() {
                                self.will_emit(Token::Character('<'));
                                self.will_emit(Token::Character('/'));
                                self.emit_temp_buffer();
                                self.reconsume_in(State::RAWTEXT);
                            } else {
                                self.switch_to(State::BeforeAttributeName);
                            }
                        }
                        Char::ch('/') => {
                            if !self.is_end_tag_appropriate() {
                                self.will_emit(Token::Character('<'));
                                self.will_emit(Token::Character('/'));
                                self.emit_temp_buffer();
                                self.reconsume_in(State::RAWTEXT);
                            } else {
                                self.switch_to(State::SelfClosingStartTag);
                            }
                        }
                        Char::ch('>') => {
                            if !self.is_end_tag_appropriate() {
                                self.will_emit(Token::Character('<'));
                                self.will_emit(Token::Character('/'));
                                self.emit_temp_buffer();
                                self.reconsume_in(State::RAWTEXT);
                            } else {
                                self.switch_to(State::Data);
                                return self.emit_current_token();
                            }
                        }
                        Char::ch(c) if c.is_ascii_uppercase() => {
                            self.append_character_to_tag_name(c.to_ascii_lowercase());
                            self.temp_buffer.push(self.current_character);
                        }
                        Char::ch(c) if c.is_ascii_lowercase() => {
                            self.append_character_to_tag_name(self.current_character);
                            self.temp_buffer.push(self.current_character);
                        }
                        _ => {
                            self.will_emit(Token::Character('<'));
                            self.will_emit(Token::Character('/'));
                            self.emit_temp_buffer();
                            self.reconsume_in(State::RAWTEXT);
                        }
                    }
                }
                State::ScriptDataLessThanSign => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('/') => {
                            self.temp_buffer.clear();
                            self.switch_to(State::ScriptDataEndTagOpen);
                        }
                        Char::ch('!') => {
                            self.will_emit(Token::Character('<'));
                            self.will_emit(Token::Character('!'));
                            self.switch_to(State::ScriptDataEscapeStart);
                        }
                        _ => {
                            self.will_emit(Token::Character('<'));
                            self.reconsume_in(State::ScriptData);
                        }
                    }
                }
                State::ScriptDataEndTagOpen => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch(c) if c.is_ascii_alphabetic() => {
                            self.new_token(Token::new_end_tag());
                            self.reconsume_in(State::ScriptDataEndTagName);
                        }
                        _ => {
                            self.will_emit(Token::Character('<'));
                            self.will_emit(Token::Character('/'));
                            self.reconsume_in(State::ScriptData);
                        }
                    }
                }
                State::ScriptDataEndTagName => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace => {
                            if !self.is_end_tag_appropriate() {
                                self.will_emit(Token::Character('<'));
                                self.will_emit(Token::Character('/'));
                                self.emit_temp_buffer();
                                self.reconsume_in(State::ScriptData);
                            } else {
                                self.switch_to(State::BeforeAttributeName);
                            }
                        }
                        Char::ch('/') => {
                            if !self.is_end_tag_appropriate() {
                                self.will_emit(Token::Character('<'));
                                self.will_emit(Token::Character('/'));
                                self.emit_temp_buffer();
                                self.reconsume_in(State::ScriptData);
                            } else {
                                self.switch_to(State::SelfClosingStartTag);
                            }
                        }
                        Char::ch('>') => {
                            if !self.is_end_tag_appropriate() {
                                self.will_emit(Token::Character('<'));
                                self.will_emit(Token::Character('/'));
                                self.emit_temp_buffer();
                                self.reconsume_in(State::ScriptData);
                            } else {
                                self.switch_to(State::Data);
                                return self.emit_current_token();
                            }
                        }
                        Char::ch(c) if c.is_ascii_uppercase() => {
                            self.append_character_to_tag_name(c.to_ascii_lowercase());
                            self.temp_buffer.push(self.current_character);
                        }
                        Char::ch(c) if c.is_ascii_lowercase() => {
                            self.append_character_to_tag_name(self.current_character);
                            self.temp_buffer.push(self.current_character);
                        }
                        _ => {
                            self.will_emit(Token::Character('<'));
                            self.will_emit(Token::Character('/'));
                            self.emit_temp_buffer();
                            self.reconsume_in(State::ScriptData);
                        }
                    }
                }
                State::ScriptDataEscapeStart => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('-') => {
                            self.switch_to(State::ScriptDataEscapeStartDash);
                        }
                        _ => {
                            self.reconsume_in(State::ScriptData);
                        }
                    }
                }
                State::ScriptDataEscapeStartDash => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('-') => {
                            self.switch_to(State::ScriptDataEscapedDashDash);
                        }
                        _ => {
                            self.reconsume_in(State::ScriptData);
                        }
                    }
                }
                State::ScriptDataEscaped => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('-') => {
                            self.switch_to(State::ScriptDataEscapedDash);
                            return self.emit_char('-');
                        }
                        Char::ch('<') => {
                            self.switch_to(State::ScriptDataEscapedLessThanSign);
                        }
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            return self.emit_char(REPLACEMENT_CHARACTER);
                        }
                        Char::eof => {
                            emit_error!("eof-in-script-html-comment-like-text");
                            return self.emit_eof();
                        }
                        _ => {
                            return self.emit_current_char();
                        }
                    }
                }
                State::ScriptDataEscapedDash => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('-') => {
                            self.switch_to(State::ScriptDataEscapedDashDash);
                            return self.emit_char('-');
                        }
                        Char::ch('<') => {
                            self.switch_to(State::ScriptDataEscapedLessThanSign);
                        }
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            self.switch_to(State::ScriptDataEscaped);
                            return self.emit_char(REPLACEMENT_CHARACTER);
                        }
                        Char::eof => {
                            emit_error!("eof-in-script-html-comment-like-text");
                            return self.emit_eof();
                        }
                        _ => {
                            self.switch_to(State::ScriptDataEscaped);
                            return self.emit_current_char();
                        }
                    }
                }
                State::ScriptDataEscapedDashDash => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('-') => {
                            return self.emit_char('-');
                        }
                        Char::ch('<') => {
                            self.switch_to(State::ScriptDataEscapedLessThanSign);
                        }
                        Char::ch('>') => {
                            self.switch_to(State::ScriptData);
                            return self.emit_char('>');
                        }
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            self.switch_to(State::ScriptDataEscaped);
                            return self.emit_char(REPLACEMENT_CHARACTER);
                        }
                        Char::eof => {
                            emit_error!("eof-in-script-html-comment-like-text");
                            return self.emit_eof();
                        }
                        _ => {
                            self.switch_to(State::ScriptDataEscaped);
                            return self.emit_current_char();
                        }
                    }
                }
                State::ScriptDataEscapedLessThanSign => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('/') => {
                            self.temp_buffer.clear();
                            self.switch_to(State::ScriptDataEscapedEndTagOpen);
                        }
                        Char::ch(c) if c.is_ascii_alphabetic() => {
                            self.temp_buffer.clear();
                            self.will_emit(Token::Character('<'));
                            self.reconsume_in(State::ScriptDataDoubleEscapeStart);
                        }
                        _ => {
                            self.will_emit(Token::Character('<'));
                            self.reconsume_in(State::ScriptDataEscaped);
                        }
                    }
                }
                State::ScriptDataEscapedEndTagOpen => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch(c) if c.is_ascii_alphabetic() => {
                            self.new_token(Token::new_end_tag());
                            self.reconsume_in(State::ScriptDataEscapedEndTagName);
                        }
                        _ => {
                            self.will_emit(Token::Character('<'));
                            self.will_emit(Token::Character('/'));
                            self.reconsume_in(State::ScriptDataEscaped);
                        }
                    }
                }
                State::ScriptDataEscapedEndTagName => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace => {
                            if !self.is_end_tag_appropriate() {
                                self.will_emit(Token::Character('<'));
                                self.will_emit(Token::Character('/'));
                                self.emit_temp_buffer();
                                self.reconsume_in(State::ScriptDataEscaped);
                            } else {
                                self.switch_to(State::BeforeAttributeName);
                            }
                        }
                        Char::ch('/') => {
                            if !self.is_end_tag_appropriate() {
                                self.will_emit(Token::Character('<'));
                                self.will_emit(Token::Character('/'));
                                self.emit_temp_buffer();
                                self.reconsume_in(State::ScriptDataEscaped);
                            } else {
                                self.switch_to(State::SelfClosingStartTag);
                            }
                        }
                        Char::ch('>') => {
                            if !self.is_end_tag_appropriate() {
                                self.will_emit(Token::Character('<'));
                                self.will_emit(Token::Character('/'));
                                self.emit_temp_buffer();
                                self.reconsume_in(State::ScriptDataEscaped);
                            } else {
                                self.switch_to(State::Data);
                                return self.emit_current_token();
                            }
                        }
                        Char::ch(c) if c.is_ascii_uppercase() => {
                            self.append_character_to_tag_name(c.to_ascii_lowercase());
                            self.temp_buffer.push(c);
                        }
                        Char::ch(c) if c.is_ascii_lowercase() => {
                            self.append_character_to_tag_name(c);
                            self.temp_buffer.push(c);
                        }
                        _ => {
                            self.will_emit(Token::Character('<'));
                            self.will_emit(Token::Character('/'));
                            self.emit_temp_buffer();
                            self.reconsume_in(State::ScriptDataEscaped);
                        }
                    }
                }
                State::ScriptDataDoubleEscapeStart => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace | Char::ch('/') | Char::ch('>') => {
                            if self.temp_buffer == "script" {
                                self.switch_to(State::ScriptDataDoubleEscaped);
                            } else {
                                self.switch_to(State::ScriptDataEscaped);
                            }
                            return self.emit_current_char();
                        }
                        Char::ch(c) if c.is_ascii_uppercase() => {
                            self.temp_buffer.push(c.to_ascii_lowercase());
                            return self.emit_current_char();
                        }
                        Char::ch(c) if c.is_ascii_lowercase() => {
                            self.temp_buffer.push(c);
                            return self.emit_current_char();
                        }
                        _ => {
                            self.reconsume_in(State::ScriptDataEscaped);
                        }
                    }
                }
                State::ScriptDataDoubleEscaped => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('-') => {
                            self.switch_to(State::ScriptDataDoubleEscapedDash);
                            return self.emit_char('-');
                        }
                        Char::ch('<') => {
                            self.switch_to(State::ScriptDataDoubleEscapedLessThanSign);
                            return self.emit_char('<');
                        }
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            return self.emit_char(REPLACEMENT_CHARACTER);
                        }
                        Char::eof => {
                            emit_error!("eof-in-script-html-comment-like-text");
                            return self.emit_eof();
                        }
                        _ => {
                            return self.emit_current_char();
                        }
                    }
                }
                State::ScriptDataDoubleEscapedDash => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('-') => {
                            self.switch_to(State::ScriptDataDoubleEscapedDashDash);
                            return self.emit_char('-');
                        }
                        Char::ch('<') => {
                            self.switch_to(State::ScriptDataDoubleEscapedLessThanSign);
                            return self.emit_char('<');
                        }
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            self.switch_to(State::ScriptDataDoubleEscaped);
                            return self.emit_char(REPLACEMENT_CHARACTER);
                        }
                        Char::eof => {
                            emit_error!("eof-in-script-html-comment-like-text");
                            return self.emit_eof();
                        }
                        _ => {
                            self.switch_to(State::ScriptDataDoubleEscaped);
                            return self.emit_current_char();
                        }
                    }
                }
                State::ScriptDataDoubleEscapedDashDash => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('-') => {
                            return self.emit_char('-');
                        }
                        Char::ch('<') => {
                            self.switch_to(State::ScriptDataDoubleEscapedLessThanSign);
                            return self.emit_char('<');
                        }
                        Char::ch('>') => {
                            self.switch_to(State::ScriptData);
                            return self.emit_char('>');
                        }
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            self.switch_to(State::ScriptDataDoubleEscaped);
                            return self.emit_char(REPLACEMENT_CHARACTER);
                        }
                        Char::eof => {
                            emit_error!("eof-in-script-html-comment-like-text");
                            return self.emit_eof();
                        }
                        _ => {
                            self.switch_to(State::ScriptDataDoubleEscaped);
                            return self.emit_current_char();
                        }
                    }
                }
                State::ScriptDataDoubleEscapedLessThanSign => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('/') => {
                            self.temp_buffer.clear();
                            self.switch_to(State::ScriptDataDoubleEscapeEnd);
                            return self.emit_char('/');
                        }
                        _ => {
                            self.reconsume_in(State::ScriptDataDoubleEscaped);
                        }
                    }
                }
                State::ScriptDataDoubleEscapeEnd => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace | Char::ch('/') | Char::ch('>') => {
                            if self.temp_buffer == "script" {
                                self.switch_to(State::ScriptDataEscaped);
                            } else {
                                self.switch_to(State::ScriptDataDoubleEscaped);
                            }
                            return self.emit_current_char();
                        }
                        Char::ch(c) if c.is_ascii_uppercase() => {
                            self.temp_buffer.push(c.to_ascii_lowercase());
                            return self.emit_current_char();
                        }
                        Char::ch(c) if c.is_ascii_lowercase() => {
                            self.temp_buffer.push(c);
                            return self.emit_current_char();
                        }
                        _ => {
                            self.reconsume_in(State::ScriptDataDoubleEscaped);
                        }
                    }
                }
                State::BeforeAttributeName => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace => continue,
                        Char::ch('/') | Char::ch('>') | Char::eof => {
                            self.reconsume_in(State::AfterAttributeName);
                        }
                        Char::ch('=') => {
                            emit_error!("unexpected-equals-sign-before-attribute-name");
                            let mut attribute = Attribute::new();
                            attribute.name.push(self.current_character);
                            self.new_attribute(attribute);
                            self.switch_to(State::AttributeName);
                        }
                        _ => {
                            let attribute = Attribute::new();
                            self.new_attribute(attribute);
                            self.reconsume_in(State::AttributeName);
                        }
                    }
                }
                State::AttributeName => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace | Char::ch('/') | Char::ch('>') | Char::eof => {
                            self.reconsume_in(State::AfterAttributeName);
                        }
                        Char::ch('=') => {
                            self.switch_to(State::BeforeAttributeValue);
                        }
                        Char::ch(c) if c.is_ascii_uppercase() => {
                            self.append_character_to_attribute_name(c.to_ascii_lowercase());
                        }
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            self.append_character_to_attribute_name(REPLACEMENT_CHARACTER);
                        }
                        Char::ch('"') | Char::ch('\'') | Char::ch('<') => {
                            emit_error!("unexpected-character-in-attribute-name");
                            self.append_character_to_attribute_name(self.current_character);
                        }
                        _ => {
                            self.append_character_to_attribute_name(self.current_character);
                        }
                    }
                }
                State::AfterAttributeName => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace => continue,
                        Char::ch('/') => {
                            self.switch_to(State::SelfClosingStartTag);
                        }
                        Char::ch('=') => {
                            self.switch_to(State::BeforeAttributeValue);
                        }
                        Char::ch('>') => {
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-tag");
                            return self.emit_eof();
                        }
                        _ => {
                            let attribute = Attribute::new();
                            self.new_attribute(attribute);
                            self.reconsume_in(State::AttributeName);
                        }
                    }
                }
                State::BeforeAttributeValue => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace => continue,
                        Char::ch('"') => {
                            self.switch_to(State::AttributeValueDoubleQuoted);
                        }
                        Char::ch('\'') => {
                            self.switch_to(State::AttributeValueSingleQuoted);
                        }
                        Char::ch('>') => {
                            emit_error!("missing-attribute-value");
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        _ => {
                            self.reconsume_in(State::AttributeValueUnQuoted);
                        }
                    }
                }
                State::AttributeValueDoubleQuoted => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('"') => {
                            self.switch_to(State::AfterAttributeValueQuoted);
                        }
                        Char::ch('&') => {
                            self.return_state = Some(State::AttributeValueDoubleQuoted);
                            self.switch_to(State::CharacterReference);
                        }
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            self.append_character_to_attribute_value(REPLACEMENT_CHARACTER);
                        }
                        Char::eof => {
                            emit_error!("eof-in-tag");
                            return self.emit_eof();
                        }
                        _ => {
                            self.append_character_to_attribute_value(self.current_character);
                        }
                    }
                }
                State::AttributeValueSingleQuoted => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('\'') => {
                            self.switch_to(State::AfterAttributeValueQuoted);
                        }
                        Char::ch('&') => {
                            self.return_state = Some(State::AttributeValueSingleQuoted);
                            self.switch_to(State::CharacterReference);
                        }
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            self.append_character_to_attribute_value(REPLACEMENT_CHARACTER);
                        }
                        Char::eof => {
                            emit_error!("eof-in-tag");
                            return self.emit_eof();
                        }
                        _ => {
                            self.append_character_to_attribute_value(self.current_character);
                        }
                    }
                }
                State::AttributeValueUnQuoted => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace => {
                            self.switch_to(State::BeforeAttributeName);
                        }
                        Char::ch('&') => {
                            self.return_state = Some(State::AttributeValueUnQuoted);
                            self.switch_to(State::CharacterReference);
                        }
                        Char::ch('>') => {
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            self.append_character_to_attribute_value(REPLACEMENT_CHARACTER);
                        }
                        Char::ch('"')
                        | Char::ch('\'')
                        | Char::ch('<')
                        | Char::ch('=')
                        | Char::ch('`') => {
                            emit_error!("unexpected-character-in-unquoted-attribute-value");
                            self.append_character_to_attribute_value(self.current_character);
                        }
                        Char::eof => {
                            emit_error!("eof-in-tag");
                            return self.emit_eof();
                        }
                        _ => {
                            self.append_character_to_attribute_value(self.current_character);
                        }
                    }
                }
                State::AfterAttributeValueQuoted => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace => {
                            self.switch_to(State::BeforeAttributeName);
                        }
                        Char::ch('/') => {
                            self.switch_to(State::SelfClosingStartTag);
                        }
                        Char::ch('>') => {
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-tag");
                            return self.emit_eof();
                        }
                        _ => {
                            emit_error!("missing-whitespace-between-attributes");
                            self.reconsume_in(State::BeforeAttributeName);
                        }
                    }
                }
                State::SelfClosingStartTag => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('>') => {
                            let tag = self.current_token.as_mut().unwrap();
                            if let Token::Tag {
                                ref mut self_closing,
                                ..
                            } = tag
                            {
                                *self_closing = true;
                            }
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-tag");
                            return self.emit_eof();
                        }
                        _ => {
                            emit_error!("unexpected-solidus-in-tag");
                            self.reconsume_in(State::BeforeAttributeName);
                        }
                    }
                }
                State::BogusComment => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('>') => {
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            self.append_character_to_token_data(REPLACEMENT_CHARACTER);
                        }
                        _ => {
                            self.append_character_to_token_data(self.current_character);
                        }
                    }
                }
                State::MarkupDeclarationOpen => {
                    if self.consume_if_match("--", false) {
                        self.new_token(Token::new_comment(""));
                        self.switch_to(State::CommentStart);
                    } else if self.consume_if_match("doctype", true) {
                        self.switch_to(State::DOCTYPE);
                    } else if self.consume_if_match("[CDATA[", false) {
                        // TODO: implement this
                        unimplemented!();
                    } else {
                        emit_error!("incorrectly-opened-comment");
                        self.new_token(Token::new_comment(""));
                        self.switch_to(State::BogusComment);
                    }
                }
                State::CommentStart => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('-') => {
                            self.switch_to(State::CommentStartDash);
                        }
                        Char::ch('>') => {
                            emit_error!("abrupt-closing-of-empty-comment");
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        _ => {
                            self.reconsume_in(State::Comment);
                        }
                    }
                }
                State::CommentStartDash => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('-') => {
                            self.switch_to(State::CommentEnd);
                        }
                        Char::ch('>') => {
                            emit_error!("abrupt-closing-of-empty-comment");
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-comment");
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            self.append_character_to_token_data('-');
                            self.reconsume_in(State::Comment);
                        }
                    }
                }
                State::Comment => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('<') => {
                            self.append_character_to_token_data(self.current_character);
                            self.switch_to(State::CommentLessThanSign);
                        }
                        Char::ch('-') => {
                            self.switch_to(State::CommentEndDash);
                        }
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            self.append_character_to_token_data(REPLACEMENT_CHARACTER);
                        }
                        Char::eof => {
                            emit_error!("eof-in-comment");
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            self.append_character_to_token_data(self.current_character);
                        }
                    }
                }
                State::CommentLessThanSign => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('!') => {
                            self.append_character_to_token_data(self.current_character);
                            self.switch_to(State::CommentLessThanSignBang);
                        }
                        Char::ch('<') => {
                            self.append_character_to_token_data(self.current_character);
                        }
                        _ => {
                            self.reconsume_in(State::Comment);
                        }
                    }
                }
                State::CommentLessThanSignBang => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('-') => {
                            self.switch_to(State::CommentLessThanSignBangDash);
                        }
                        _ => {
                            self.reconsume_in(State::Comment);
                        }
                    }
                }
                State::CommentLessThanSignBangDash => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('-') => {
                            self.switch_to(State::CommentLessThanSignBangDashDash);
                        }
                        _ => {
                            self.reconsume_in(State::CommentEndDash);
                        }
                    }
                }
                State::CommentLessThanSignBangDashDash => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('>') | Char::eof => {
                            self.reconsume_in(State::CommentEnd);
                        }
                        _ => {
                            emit_error!("nested-comment");
                            self.reconsume_in(State::CommentEnd);
                        }
                    }
                }
                State::CommentEndDash => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('-') => {
                            self.switch_to(State::CommentEnd);
                        }
                        Char::eof => {
                            emit_error!("eof-in-comment");
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            self.append_character_to_token_data('-');
                            self.reconsume_in(State::Comment);
                        }
                    }
                }
                State::CommentEnd => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('>') => {
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::ch('!') => {
                            self.switch_to(State::CommentEndBang);
                        }
                        Char::ch('-') => {
                            self.append_character_to_token_data('-');
                        }
                        Char::eof => {
                            emit_error!("eof-in-comment");
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            self.append_character_to_token_data('-');
                            self.append_character_to_token_data('-');
                            self.reconsume_in(State::Comment);
                        }
                    }
                }
                State::CommentEndBang => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('-') => {
                            self.append_character_to_token_data('-');
                            self.append_character_to_token_data('-');
                            self.append_character_to_token_data('!');
                            self.switch_to(State::CommentEndDash);
                        }
                        Char::ch('>') => {
                            emit_error!("incorrectly-closed-comment");
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-comment");
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            self.append_character_to_token_data('-');
                            self.append_character_to_token_data('-');
                            self.append_character_to_token_data('!');
                            self.reconsume_in(State::Comment);
                        }
                    }
                }
                State::DOCTYPE => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace => {
                            self.switch_to(State::BeforeDOCTYPEName);
                        }
                        Char::ch('>') => {
                            self.reconsume_in(State::BeforeDOCTYPEName);
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let mut token = Token::new_doctype();
                            token.set_force_quirks(true);
                            self.new_token(token);
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            emit_error!("missing-whitespace-before-doctype-name");
                            self.reconsume_in(State::BeforeDOCTYPEName);
                        }
                    }
                }
                State::BeforeDOCTYPEName => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace => continue,
                        Char::ch(c) if c.is_ascii_uppercase() => {
                            let mut token = Token::new_doctype();
                            if let Token::DOCTYPE { ref mut name, .. } = token {
                                let mut new_name = String::new();
                                new_name.push(c.to_ascii_lowercase());
                                *name = Some(new_name);
                            }
                            self.new_token(token);
                            self.switch_to(State::DOCTYPEName);
                        }
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            let mut token = Token::new_doctype();
                            if let Token::DOCTYPE { ref mut name, .. } = token {
                                let mut new_name = String::new();
                                new_name.push(REPLACEMENT_CHARACTER);
                                *name = Some(new_name);
                            }
                            self.new_token(token);
                            self.switch_to(State::DOCTYPEName);
                        }
                        Char::ch('>') => {
                            emit_error!("missing-doctype-name");
                            let mut token = Token::new_doctype();
                            token.set_force_quirks(true);
                            self.new_token(token);
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let mut token = Token::new_doctype();
                            token.set_force_quirks(true);
                            self.new_token(token);
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            let mut token = Token::new_doctype();
                            if let Token::DOCTYPE { ref mut name, .. } = token {
                                let mut new_name = String::new();
                                new_name.push(self.current_character);
                                *name = Some(new_name);
                            }
                            self.new_token(token);
                            self.switch_to(State::DOCTYPEName);
                        }
                    }
                }
                State::DOCTYPEName => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace => {
                            self.switch_to(State::AfterDOCTYPEName);
                        }
                        Char::ch('>') => {
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::ch(c) if c.is_ascii_uppercase() => {
                            self.append_character_to_doctype_name(c.to_ascii_lowercase());
                        }
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            self.append_character_to_doctype_name(REPLACEMENT_CHARACTER);
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            self.append_character_to_doctype_name(self.current_character);
                        }
                    }
                }
                State::AfterDOCTYPEName => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace => continue,
                        Char::ch('>') => {
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            if self.consume_from_current_if_match("PUBLIC", true) {
                                self.switch_to(State::AfterDOCTYPEPublicKeyword);
                            } else if self.consume_from_current_if_match("SYSTEM", true) {
                                self.switch_to(State::AfterDOCTYPESystemKeyword);
                            } else {
                                emit_error!("invalid-character-sequence-after-doctype-name");
                                let token = self.current_token.as_mut().unwrap();
                                if let Token::DOCTYPE {
                                    ref mut force_quirks,
                                    ..
                                } = token
                                {
                                    *force_quirks = true;
                                }
                                self.reconsume_in(State::BogusDOCTYPE);
                            }
                        }
                    }
                }
                State::AfterDOCTYPEPublicKeyword => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace => {
                            self.switch_to(State::BeforeDOCTYPEPublicIdentifier);
                        }
                        Char::ch('"') => {
                            emit_error!("missing-whitespace-after-doctype-public-keyword");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut public_identifier,
                                ..
                            } = token
                            {
                                *public_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPEPublicIdentifierDoubleQuoted);
                        }
                        Char::ch('\'') => {
                            emit_error!("missing-whitespace-after-doctype-public-keyword");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut public_identifier,
                                ..
                            } = token
                            {
                                *public_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPEPublicIdentifierSingleQuoted);
                        }
                        Char::ch('>') => {
                            emit_error!("missing-doctype-public-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            emit_error!("missing-quote-before-doctype-public-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.reconsume_in(State::BogusDOCTYPE);
                        }
                    }
                }
                State::BeforeDOCTYPEPublicIdentifier => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace => continue,
                        Char::ch('"') => {
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut public_identifier,
                                ..
                            } = token
                            {
                                *public_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPEPublicIdentifierDoubleQuoted);
                        }
                        Char::ch('\'') => {
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut public_identifier,
                                ..
                            } = token
                            {
                                *public_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPEPublicIdentifierSingleQuoted);
                        }
                        Char::ch('>') => {
                            emit_error!("missing-doctype-public-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            emit_error!("missing-quote-before-doctype-public-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.reconsume_in(State::BogusDOCTYPE);
                        }
                    }
                }
                State::DOCTYPEPublicIdentifierDoubleQuoted => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('"') => {
                            self.switch_to(State::AfterDOCTYPEPublicIdentifier);
                        }
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            self.append_character_to_doctype_public_identifier(
                                REPLACEMENT_CHARACTER,
                            );
                        }
                        Char::ch('>') => {
                            emit_error!("abrupt-doctype-public-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            self.append_character_to_doctype_public_identifier(
                                self.current_character,
                            );
                        }
                    }
                }
                State::DOCTYPEPublicIdentifierSingleQuoted => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('\'') => {
                            self.switch_to(State::AfterDOCTYPEPublicIdentifier);
                        }
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            self.append_character_to_doctype_public_identifier(
                                REPLACEMENT_CHARACTER,
                            );
                        }
                        Char::ch('>') => {
                            emit_error!("abrupt-doctype-public-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            self.append_character_to_doctype_public_identifier(
                                self.current_character,
                            );
                        }
                    }
                }
                State::AfterDOCTYPEPublicIdentifier => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace => {
                            self.switch_to(State::BetweenDOCTYPEPublicAndSystemIdentifiers);
                        }
                        Char::ch('>') => {
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::ch('"') => {
                            emit_error!(
                                "missing-whitespace-between-doctype-public-and-system-identifiers"
                            );
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut system_identifier,
                                ..
                            } = token
                            {
                                *system_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPESytemIdentifierDoubleQuoted);
                        }
                        Char::ch('\'') => {
                            emit_error!(
                                "missing-whitespace-between-doctype-public-and-system-identifiers"
                            );
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut system_identifier,
                                ..
                            } = token
                            {
                                *system_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPESytemIdentifierSingleQuoted);
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            emit_error!("missing-quote-before-doctype-system-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.reconsume_in(State::BogusDOCTYPE);
                        }
                    }
                }
                State::BetweenDOCTYPEPublicAndSystemIdentifiers => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace => continue,
                        Char::ch('>') => {
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::ch('"') => {
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut system_identifier,
                                ..
                            } = token
                            {
                                *system_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPESytemIdentifierDoubleQuoted);
                        }
                        Char::ch('\'') => {
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut system_identifier,
                                ..
                            } = token
                            {
                                *system_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPESytemIdentifierSingleQuoted);
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            emit_error!("missing-quote-before-doctype-system-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.reconsume_in(State::BogusDOCTYPE);
                        }
                    }
                }
                State::AfterDOCTYPESystemKeyword => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace => {
                            self.switch_to(State::BeforeDOCTYPESystemIdentifier);
                        }
                        Char::ch('"') => {
                            emit_error!("missing-whitespace-after-doctype-system-keyword");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut system_identifier,
                                ..
                            } = token
                            {
                                *system_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPESytemIdentifierDoubleQuoted);
                        }
                        Char::ch('\'') => {
                            emit_error!("missing-whitespace-after-doctype-system-keyword");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut system_identifier,
                                ..
                            } = token
                            {
                                *system_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPESytemIdentifierSingleQuoted);
                        }
                        Char::ch('>') => {
                            emit_error!("missing-doctype-system-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            emit_error!("missing-quote-before-doctype-system-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.reconsume_in(State::BogusDOCTYPE);
                        }
                    }
                }
                State::BeforeDOCTYPESystemIdentifier => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace => continue,
                        Char::ch('"') => {
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut system_identifier,
                                ..
                            } = token
                            {
                                *system_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPESytemIdentifierDoubleQuoted);
                        }
                        Char::ch('\'') => {
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut system_identifier,
                                ..
                            } = token
                            {
                                *system_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPESytemIdentifierSingleQuoted);
                        }
                        Char::ch('>') => {
                            emit_error!("missing-doctype-system-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            emit_error!("missing-quote-before-doctype-system-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.reconsume_in(State::BogusDOCTYPE);
                        }
                    }
                }
                State::DOCTYPESytemIdentifierDoubleQuoted => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('"') => {
                            self.switch_to(State::AfterDOCTYPESystemIdentifier);
                        }
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            self.append_character_to_doctype_system_identifier(
                                REPLACEMENT_CHARACTER,
                            );
                        }
                        Char::ch('>') => {
                            emit_error!("abrupt-doctype-system-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            self.append_character_to_doctype_system_identifier(
                                self.current_character,
                            );
                        }
                    }
                }
                State::DOCTYPESytemIdentifierSingleQuoted => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('\'') => {
                            self.switch_to(State::AfterDOCTYPESystemIdentifier);
                        }
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            self.append_character_to_doctype_system_identifier(
                                REPLACEMENT_CHARACTER,
                            );
                        }
                        Char::ch('>') => {
                            emit_error!("abrupt-doctype-system-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            self.append_character_to_doctype_system_identifier(
                                self.current_character,
                            );
                        }
                    }
                }
                State::AfterDOCTYPESystemIdentifier => {
                    let ch = self.consume_next();
                    match ch {
                        Char::whitespace => continue,
                        Char::ch('>') => {
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE {
                                ref mut force_quirks,
                                ..
                            } = token
                            {
                                *force_quirks = true;
                            }
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            emit_error!("unexpected-character-after-doctype-system-identifier");
                            self.reconsume_in(State::BogusDOCTYPE);
                        }
                    }
                }
                State::BogusDOCTYPE => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('>') => {
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            continue;
                        }
                        Char::eof => {
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            continue;
                        }
                    }
                }
                State::CDATASection => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch(']') => {
                            self.switch_to(State::CDATASectionBracket);
                        }
                        Char::eof => {
                            emit_error!("eof-in-cdata");
                            return self.emit_eof();
                        }
                        _ => {
                            return self.emit_current_char();
                        }
                    }
                }
                State::CDATASectionBracket => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch(']') => {
                            self.switch_to(State::CDATASectionEnd);
                        }
                        _ => {
                            self.will_emit(Token::Character(']'));
                            self.reconsume_in(State::CDATASection);
                        }
                    }
                }
                State::CDATASectionEnd => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch(']') => {
                            return self.emit_char(']');
                        }
                        Char::ch('>') => {
                            self.switch_to(State::Data);
                        }
                        _ => {
                            self.will_emit(Token::Character(']'));
                            self.will_emit(Token::Character(']'));
                            self.reconsume_in(State::CDATASection);
                        }
                    }
                }
                State::CharacterReference => {
                    let ch = self.consume_next();
                    self.temp_buffer.clear();
                    self.temp_buffer.push('&');
                    match ch {
                        Char::ch(c) if c.is_ascii_alphanumeric() => {
                            self.reconsume_in(State::NamedCharacterReference);
                        }
                        Char::ch('#') => {
                            self.temp_buffer.push(self.current_character);
                            self.switch_to(State::NumericCharacterReference);
                        }
                        _ => {
                            self.flush_code_points_consumed_as_a_character_reference();
                            self.reconsume_in_return_state();
                        }
                    }
                }
                State::NamedCharacterReference => {
                    let current_str = format!("{}{}", self.current_character, self.input.as_str());
                    let mut match_result: Option<(&str, u32, u32)> = None;
                    let mut max_len: usize = 0;

                    // TODO: optimize matching
                    for entity in ENTITIES.iter() {
                        let entity_name = entity.0;
                        let entity_len = entity_name.len();
                        if current_str.starts_with(entity_name) {
                            if entity_len > max_len {
                                max_len = entity_len;
                                match_result = Some(*entity);
                            }
                        }
                    }

                    if let Some(result) = match_result {
                        let (entity_name, charcode_1, charcode_2) = result;
                        for c in entity_name.chars() {
                            self.consume_next();
                            self.temp_buffer.push(c);
                        }

                        let last_match_ch = self.current_character;
                        if self.is_character_part_of_attribute() && last_match_ch != ';' {
                            let next_ch = current_str.chars().nth(max_len - 1);
                            if let Some(next_ch) = next_ch {
                                if next_ch == '=' || next_ch.is_ascii_alphanumeric() {
                                    self.flush_code_points_consumed_as_a_character_reference();
                                    self.switch_to_return_state();
                                    continue;
                                }
                            }
                        }

                        if last_match_ch != ';' {
                            emit_error!("missing-semicolon-after-character-reference");
                        }

                        self.temp_buffer.clear();
                        self.temp_buffer
                            .push(std::char::from_u32(charcode_1).unwrap());
                        if charcode_2 != 0 {
                            self.temp_buffer
                                .push(std::char::from_u32(charcode_2).unwrap());
                        }
                        self.flush_code_points_consumed_as_a_character_reference();
                        self.switch_to_return_state();
                    } else {
                        self.flush_code_points_consumed_as_a_character_reference();
                        self.switch_to(State::AmbiguousAmpersand);
                    }
                }
                State::AmbiguousAmpersand => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch(c) if c.is_ascii_alphanumeric() => {
                            if self.is_character_part_of_attribute() {
                                self.append_character_to_attribute_value(c);
                            } else {
                                return self.emit_current_char();
                            }
                        }
                        Char::ch(';') => {
                            emit_error!("unknown-named-character-reference");
                            self.reconsume_in_return_state();
                        }
                        _ => {
                            self.reconsume_in_return_state();
                        }
                    }
                }
                State::NumericCharacterReference => {
                    self.character_reference_code = 0;
                    let ch = self.consume_next();
                    match ch {
                        Char::ch('x') | Char::ch('X') => {
                            self.temp_buffer.push(self.current_character);
                            self.switch_to(State::HexadecimalCharacterReferenceStart);
                        }
                        _ => {
                            self.reconsume_in(State::DecimalCharacterReferenceStart);
                        }
                    }
                }
                State::HexadecimalCharacterReferenceStart => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch(c) if c.is_ascii_hexdigit() => {
                            self.reconsume_in(State::HexadecimalCharacterReference);
                        }
                        _ => {
                            emit_error!("absence-of-digits-in-numeric-character-reference");
                            self.flush_code_points_consumed_as_a_character_reference();
                            self.reconsume_in_return_state();
                        }
                    }
                }
                State::DecimalCharacterReferenceStart => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch(c) if c.is_ascii_digit() => {
                            self.reconsume_in(State::DecimalCharacterReference);
                        }
                        _ => {
                            emit_error!("absence-of-digits-in-numeric-character-reference");
                            self.flush_code_points_consumed_as_a_character_reference();
                            self.reconsume_in_return_state();
                        }
                    }
                }
                State::HexadecimalCharacterReference => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch(c) if c.is_ascii_digit() => {
                            self.character_reference_code *= 16;
                            if let Some(d) = self.current_character.to_digit(10) {
                                self.character_reference_code += d;
                            } else {
                                emit_error!("Can't convert current character to digit");
                            }
                        }
                        Char::ch(c) if c.is_ascii_hexdigit() => {
                            self.character_reference_code *= 16;
                            if let Some(d) = self.current_character.to_digit(16) {
                                self.character_reference_code += d;
                            } else {
                                emit_error!("Can't convert current character to digit");
                            }
                        }
                        Char::ch(';') => {
                            self.switch_to(State::NumericCharacterReferenceEnd);
                        }
                        _ => {
                            emit_error!("missing-semicolon-after-character-reference");
                            self.reconsume_in(State::NumericCharacterReferenceEnd);
                        }
                    }
                }
                State::DecimalCharacterReference => {
                    let ch = self.consume_next();
                    match ch {
                        Char::ch(c) if c.is_ascii_digit() => {
                            self.character_reference_code *= 10;
                            if let Some(d) = self.current_character.to_digit(10) {
                                self.character_reference_code += d;
                            } else {
                                emit_error!("Can't convert current character to digit");
                            }
                        }
                        Char::ch(';') => {
                            self.switch_to(State::NumericCharacterReferenceEnd);
                        }
                        _ => {
                            emit_error!("missing-semicolon-after-character-reference");
                            self.reconsume_in(State::NumericCharacterReferenceEnd);
                        }
                    }
                }
                State::NumericCharacterReferenceEnd => {
                    let code = self.character_reference_code;
                    if code == 0x00 {
                        emit_error!("null-character-reference");
                        self.character_reference_code = 0xFFFD;
                    }
                    if code > 0x10FFFF {
                        emit_error!("character-reference-outside-unicode-range");
                        self.character_reference_code = 0xFFFD;
                    }
                    if is_surrogate(code) {
                        emit_error!("surrogate-character-reference");
                        self.character_reference_code = 0xFFFD;
                    }
                    if is_nonecharacter(code) {
                        emit_error!("noncharacter-character-reference");
                    }
                    if code == 0x0D || (is_control(code) && !is_whitespace(code)) {
                        emit_error!("control-character-reference");
                        if let Some(new_code) = replace_control_codes(code) {
                            self.character_reference_code = new_code;
                        }
                    }
                    let result = std::char::from_u32(self.character_reference_code)
                        .unwrap_or(REPLACEMENT_CHARACTER);
                    self.temp_buffer.clear();
                    self.temp_buffer.push(result);
                    self.flush_code_points_consumed_as_a_character_reference();
                    self.switch_to_return_state();
                }
            }
        }
    }

    fn reconsume_in_return_state(&mut self) {
        self.reconsume_in(self.return_state.clone().unwrap());
    }

    fn switch_to_return_state(&mut self) {
        self.switch_to(self.return_state.clone().unwrap());
    }

    fn new_attribute(&mut self, attribute: Attribute) {
        let token = self.current_token.as_mut().unwrap();
        if let Token::Tag {
            ref mut attributes, ..
        } = token
        {
            attributes.push(attribute);
        }
    }

    fn flush_code_points_consumed_as_a_character_reference(&mut self) {
        if self.is_character_part_of_attribute() {
            let chars: Vec<char> = self.temp_buffer.chars().collect();
            for c in chars {
                self.append_character_to_attribute_value(c);
            }
        } else {
            self.emit_temp_buffer();
        }
    }

    fn is_character_part_of_attribute(&self) -> bool {
        if let Some(return_state) = &self.return_state {
            return match return_state {
                State::AttributeValueDoubleQuoted => true,
                State::AttributeValueSingleQuoted => true,
                State::AttributeValueUnQuoted => true,
                _ => false,
            };
        }
        emit_error!("No return state found");
        false
    }

    fn emit_temp_buffer(&mut self) {
        for c in self.temp_buffer.chars() {
            self.output.push_back(Token::Character(c));
        }
    }

    fn append_character_to_doctype_name(&mut self, ch: char) {
        let token = self.current_token.as_mut().unwrap();
        if let Token::DOCTYPE { ref mut name, .. } = token {
            let name = name.as_mut().unwrap();
            name.push(ch);
        }
    }

    fn append_character_to_doctype_public_identifier(&mut self, ch: char) {
        let token = self.current_token.as_mut().unwrap();
        if let Token::DOCTYPE {
            ref mut public_identifier,
            ..
        } = token
        {
            let public_identifier = public_identifier.as_mut().unwrap();
            public_identifier.push(ch);
        }
    }

    fn append_character_to_doctype_system_identifier(&mut self, ch: char) {
        let token = self.current_token.as_mut().unwrap();
        if let Token::DOCTYPE {
            ref mut system_identifier,
            ..
        } = token
        {
            let system_identifier = system_identifier.as_mut().unwrap();
            system_identifier.push(ch);
        }
    }

    fn append_character_to_tag_name(&mut self, ch: char) {
        // better crash that hang process
        let current_tag = self.current_token.as_mut().unwrap();
        if let Token::Tag { tag_name, .. } = current_tag {
            tag_name.push(ch);
        } else {
            // hope that this never fire
            emit_error!("No tag found");
        }
    }

    fn append_character_to_token_data(&mut self, ch: char) {
        let current_tag = self.current_token.as_mut().unwrap();
        if let Token::Comment(ref mut data) = current_tag {
            data.push(ch);
        } else {
            // hope that this never fire
            emit_error!("No tag found");
        }
    }

    fn append_character_to_attribute_name(&mut self, ch: char) {
        let current_tag = self.current_token.as_mut().unwrap();
        if let Token::Tag {
            ref mut attributes, ..
        } = current_tag
        {
            let attribute = attributes.last_mut().unwrap();
            attribute.name.push(ch);
        }
    }

    fn append_character_to_attribute_value(&mut self, ch: char) {
        let current_tag = self.current_token.as_mut().unwrap();
        if let Token::Tag {
            ref mut attributes, ..
        } = current_tag
        {
            let attribute = attributes.last_mut().unwrap();
            attribute.value.push(ch);
        }
    }

    fn emit_current_token(&mut self) -> Token {
        self.will_emit(self.current_token.clone().unwrap());
        self.pop_token()
    }

    fn emit_char(&mut self, ch: char) -> Token {
        self.new_token(Token::Character(ch));
        self.emit_current_token()
    }

    fn emit_current_char(&mut self) -> Token {
        self.emit_char(self.current_character)
    }

    fn emit_eof(&mut self) -> Token {
        self.new_token(Token::EOF);
        self.emit_current_token()
    }

    fn new_token(&mut self, token: Token) {
        self.current_token = Some(token);
    }

    fn will_emit(&mut self, token: Token) {
        let mut token = token;
        if let Token::Tag {
            is_end_tag,
            ref mut attributes,
            ..
        } = token
        {
            let mut seen = HashSet::new();
            let mut remove_indexes = Vec::new();
            for (index, attribute) in attributes.iter().enumerate() {
                if seen.contains(&attribute.name) {
                    emit_error!("duplicate-attribute");
                    remove_indexes.push(index);
                } else {
                    seen.insert(attribute.name.clone());
                }
            }
            for index in remove_indexes {
                attributes.remove(index);
            }
            if !is_end_tag {
                self.last_emitted_start_tag = Some(token.clone());
            }
        }
        self.output.push_back(token);
    }

    fn is_end_tag_appropriate(&mut self) -> bool {
        if self.last_emitted_start_tag.is_none() {
            return false;
        }
        let current_tag = self.current_token.as_ref().unwrap();
        let last_start_tag = self.last_emitted_start_tag.as_ref().unwrap();

        if let Token::Tag { tag_name, .. } = current_tag {
            let current_tag_name = tag_name;
            if let Token::Tag { tag_name, .. } = last_start_tag {
                let last_tag_name = tag_name;
                return current_tag_name == last_tag_name;
            }
        }
        false
    }

    fn pop_token(&mut self) -> Token {
        self.output.pop_front().unwrap()
    }

    fn reconsume_in(&mut self, state: State) {
        self.reconsume_char = true;
        self.switch_to(state);
    }

    pub fn switch_to(&mut self, state: State) {
        if is_trace() {
            println!("Switch to: {:#?}", state);
        }
        self.state = state;
    }

    fn consume_if_match(&mut self, pattern: &str, case_insensitive: bool) -> bool {
        let mut current_str = self.input.as_str().to_owned();
        let mut pattern = pattern.to_owned();
        if case_insensitive {
            current_str = current_str.to_ascii_lowercase();
            pattern = pattern.to_ascii_lowercase();
        }
        if current_str.starts_with(&pattern) {
            for _ in 0..pattern.len() {
                self.consume_next();
            }
            return true;
        }
        false
    }

    fn consume_from_current_if_match(&mut self, pattern: &str, case_insensitive: bool) -> bool {
        let mut current_str = format!("{}{}", self.current_character, self.input.as_str());
        let mut pattern = pattern.to_owned();
        if case_insensitive {
            current_str = current_str.to_ascii_lowercase();
            pattern = pattern.to_ascii_lowercase();
        }
        if current_str.starts_with(&pattern) {
            for _ in 0..pattern.len() {
                self.consume_next();
            }
            return true;
        }
        false
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
                match c {
                    '\0' => Char::null,
                    '\t' | '\n' | '\x0C' | ' ' => Char::whitespace,
                    _ => Char::ch(c),
                }
            }
            None => Char::eof,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_comment() {
        let html = "<!--xin chao-->".to_owned();
        let mut tokenizer = Tokenizer::new(html);
        assert_eq!(
            Token::Comment("xin chao".to_owned()),
            tokenizer.next_token()
        );
    }

    #[test]
    fn parse_tag() {
        let html = "<html>".to_owned();
        let mut tokenizer = Tokenizer::new(html);
        assert_eq!(
            Token::Tag {
                tag_name: "html".to_owned(),
                self_closing: false,
                self_closing_acknowledged: false,
                attributes: Vec::new(),
                is_end_tag: false
            },
            tokenizer.next_token()
        );
    }

    #[test]
    fn parse_tag_self_closing() {
        let html = "<div />".to_owned();
        let mut tokenizer = Tokenizer::new(html);
        assert_eq!(
            Token::Tag {
                tag_name: "div".to_owned(),
                self_closing: true,
                self_closing_acknowledged: false,
                attributes: Vec::new(),
                is_end_tag: false
            },
            tokenizer.next_token()
        );
    }

    #[test]
    fn parse_tag_attribute_double_quote() {
        let html = "<div name=\"hello\" />".to_owned();
        let mut tokenizer = Tokenizer::new(html);
        assert_eq!(
            Token::Tag {
                tag_name: "div".to_owned(),
                self_closing: true,
                self_closing_acknowledged: false,
                attributes: vec![Attribute {
                    name: "name".to_owned(),
                    value: "hello".to_owned(),
                    prefix: "".to_owned(),
                    namespace: "".to_owned()
                }],
                is_end_tag: false
            },
            tokenizer.next_token()
        );
    }

    #[test]
    fn parse_tag_attribute_single_quote() {
        let html = "<div name='hello' />".to_owned();
        let mut tokenizer = Tokenizer::new(html);
        assert_eq!(
            Token::Tag {
                tag_name: "div".to_owned(),
                self_closing: true,
                self_closing_acknowledged: false,
                attributes: vec![Attribute {
                    name: "name".to_owned(),
                    value: "hello".to_owned(),
                    prefix: "".to_owned(),
                    namespace: "".to_owned()
                }],
                is_end_tag: false
            },
            tokenizer.next_token()
        );
    }

    #[test]
    fn parse_tag_attribute_unquote() {
        let html = "<div name=hello world />".to_owned();
        let mut tokenizer = Tokenizer::new(html);
        assert_eq!(
            Token::Tag {
                tag_name: "div".to_owned(),
                self_closing: true,
                self_closing_acknowledged: false,
                attributes: vec![
                    Attribute {
                        name: "name".to_owned(),
                        value: "hello".to_owned(),
                        prefix: "".to_owned(),
                        namespace: "".to_owned()
                    },
                    Attribute {
                        name: "world".to_owned(),
                        value: "".to_owned(),
                        prefix: "".to_owned(),
                        namespace: "".to_owned()
                    }
                ],
                is_end_tag: false
            },
            tokenizer.next_token()
        );
    }

    #[test]
    fn parse_doctype() {
        let html = "<!DOCTYPE html>".to_owned();
        let mut tokenizer = Tokenizer::new(html);
        assert_eq!(
            Token::DOCTYPE {
                name: Some("html".to_owned()),
                force_quirks: false,
                public_identifier: None,
                system_identifier: None
            },
            tokenizer.next_token()
        );
    }

    #[test]
    fn parse_doctype_with_identifiers() {
        let html = r#"<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01 Transitional//EN" "http://www.w3.org/TR/html4/loose.dtd">"#.to_owned();
        let mut tokenizer = Tokenizer::new(html);
        assert_eq!(
            Token::DOCTYPE {
                name: Some("html".to_owned()),
                force_quirks: false,
                public_identifier: Some("-//W3C//DTD HTML 4.01 Transitional//EN".to_owned()),
                system_identifier: Some("http://www.w3.org/TR/html4/loose.dtd".to_owned())
            },
            tokenizer.next_token()
        );
    }

    #[test]
    fn parse_decimal_character_reference() {
        let html = "&#94;".to_owned();
        let mut tokenizer = Tokenizer::new(html);
        assert_eq!(Token::Character('^'), tokenizer.next_token());
    }

    #[test]
    fn parse_hex_character_reference() {
        let html = "&#x00040;".to_owned();
        let mut tokenizer = Tokenizer::new(html);
        assert_eq!(Token::Character('@'), tokenizer.next_token());
    }

    #[test]
    fn parse_named_character_reference() {
        let html = "&AElig;".to_owned();
        let mut tokenizer = Tokenizer::new(html);
        assert_eq!(Token::Character('Æ'), tokenizer.next_token());
    }

    #[test]
    fn parse_invalid_named_character_reference() {
        let html = "&g;".to_owned();
        let mut tokenizer = Tokenizer::new(html);
        assert_eq!(Token::Character('&'), tokenizer.next_token());
    }

    #[test]
    fn parse_named_character_reference_in_string() {
        let html = "I'm &notit;".to_owned();
        let mut tokenizer = Tokenizer::new(html);
        assert_eq!(Token::Character('I'), tokenizer.next_token());
        assert_eq!(Token::Character('\''), tokenizer.next_token());
        assert_eq!(Token::Character('m'), tokenizer.next_token());
        assert_eq!(Token::Character(' '), tokenizer.next_token());
        assert_eq!(Token::Character('¬'), tokenizer.next_token());
        assert_eq!(Token::Character('i'), tokenizer.next_token());
        assert_eq!(Token::Character('t'), tokenizer.next_token());
        assert_eq!(Token::Character(';'), tokenizer.next_token());
    }

    #[test]
    fn parse_named_character_reference_in_attribute_name() {
        let html = "<br &block;=\"name\" />".to_owned();
        let mut tokenizer = Tokenizer::new(html);
        assert_eq!(
            Token::Tag {
                tag_name: "br".to_owned(),
                self_closing: true,
                is_end_tag: false,
                self_closing_acknowledged: false,
                attributes: vec![Attribute {
                    name: "&block;".to_owned(),
                    value: "name".to_owned(),
                    prefix: "".to_owned(),
                    namespace: "".to_owned()
                }]
            },
            tokenizer.next_token()
        );
    }

    #[test]
    fn parse_named_character_reference_in_attribute_value() {
        let html = "<br name=\"&block;\" />".to_owned();
        let mut tokenizer = Tokenizer::new(html);
        assert_eq!(
            Token::Tag {
                tag_name: "br".to_owned(),
                self_closing: true,
                is_end_tag: false,
                self_closing_acknowledged: false,
                attributes: vec![Attribute {
                    name: "name".to_owned(),
                    value: "█".to_owned(),
                    prefix: "".to_owned(),
                    namespace: "".to_owned()
                }]
            },
            tokenizer.next_token()
        );
    }

    #[test]
    fn parse_duplicate_attribute() {
        let html = "<div attr attr />".to_owned();
        let mut tokenizer = Tokenizer::new(html);
        assert_eq!(
            Token::Tag {
                tag_name: "div".to_owned(),
                self_closing: true,
                is_end_tag: false,
                self_closing_acknowledged: false,
                attributes: vec![Attribute {
                    name: "attr".to_owned(),
                    value: "".to_owned(),
                    prefix: "".to_owned(),
                    namespace: "".to_owned()
                }]
            },
            tokenizer.next_token()
        );
    }

    #[test]
    fn tokenize_mutliple() {
        let html = "<div><div></div><div></div></div>".to_owned();
        let mut tokenizer = Tokenizer::new(html);
        assert_eq!(
            Token::Tag {
                tag_name: "div".to_owned(),
                self_closing: false,
                is_end_tag: false,
                self_closing_acknowledged: false,
                attributes: vec![]
            },
            tokenizer.next_token()
        );
        assert_eq!(
            Token::Tag {
                tag_name: "div".to_owned(),
                self_closing: false,
                is_end_tag: false,
                self_closing_acknowledged: false,
                attributes: vec![]
            },
            tokenizer.next_token()
        );
        assert_eq!(
            Token::Tag {
                tag_name: "div".to_owned(),
                self_closing: false,
                is_end_tag: true,
                self_closing_acknowledged: false,
                attributes: vec![]
            },
            tokenizer.next_token()
        );
        assert_eq!(
            Token::Tag {
                tag_name: "div".to_owned(),
                self_closing: false,
                is_end_tag: false,
                self_closing_acknowledged: false,
                attributes: vec![]
            },
            tokenizer.next_token()
        );
        assert_eq!(
            Token::Tag {
                tag_name: "div".to_owned(),
                self_closing: false,
                is_end_tag: true,
                self_closing_acknowledged: false,
                attributes: vec![]
            },
            tokenizer.next_token()
        );
        assert_eq!(
            Token::Tag {
                tag_name: "div".to_owned(),
                self_closing: false,
                is_end_tag: true,
                self_closing_acknowledged: false,
                attributes: vec![]
            },
            tokenizer.next_token()
        );
    }
}
