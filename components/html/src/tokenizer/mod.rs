mod state;
mod token;

use std::collections::{VecDeque};
use std::str::Chars;
use std::env;
use state::State;
use token::Token;
use token::Attribute;

fn is_trace() -> bool {
    match env::var("TRACE_TOKENIZER") {
        Ok(s) => s == "true",
        _ => false
    }
}

macro_rules! trace {
    ($err:expr) => {
        println!("[ParseError][Tokenization]: {}", $err);
    }
}

macro_rules! emit_error {
    ($err:expr) => {
        if is_trace() {
            trace!($err)
        }
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
    whitespace
}

pub struct Tokenizer<'a> {
    // chars input stream for tokenizer
    input: &'a mut Chars<'a>,

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
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a mut Chars<'a>) -> Self {
        Self {
            input,
            output: VecDeque::new(),
            current_character: '\0',
            state: State::Data,
            return_state: None,
            current_token: None,
            reconsume_char: false,
            temp_buffer: String::new(),
            last_emitted_start_tag: None,
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
                        _ => return self.emit_current_char()
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
                        _ => return self.emit_current_char()
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
                        _ => return self.emit_current_char()
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
                        _ => return self.emit_current_char()
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
                        _ => return self.emit_current_char()
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
                        } Char::eof => { emit_error!("eof-in-tag");
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
                            }
                            else {
                                self.switch_to(State::BeforeAttributeName);
                            }
                        }
                        Char::ch('/') => {
                            if !self.is_end_tag_appropriate() {
                                self.will_emit(Token::Character('<'));
                                self.will_emit(Token::Character('/'));
                                self.emit_temp_buffer();
                                self.reconsume_in(State::ScriptDataEscaped);
                            }
                            else {
                                self.switch_to(State::SelfClosingStartTag);
                            }
                        }
                        Char::ch('>') => {
                            if !self.is_end_tag_appropriate() {
                                self.will_emit(Token::Character('<'));
                                self.will_emit(Token::Character('/'));
                                self.emit_temp_buffer();
                                self.reconsume_in(State::ScriptDataEscaped);
                            }
                            else {
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
                        Char::ch('"') => {
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
                        Char::ch('"') | Char::ch('\'') | Char::ch('<') | Char::ch('=') | Char::ch('`') =>  {
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
                            if let Token::Tag { ref mut self_closing, .. } = tag {
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
                            // TODO: replace with char::REPLACEMENT_CHARACTER when stable
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
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
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
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
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
                                if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
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
                            if let Token::DOCTYPE { ref mut public_identifier, .. } = token {
                                *public_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPEPublicIdentifierDoubleQuoted);
                        }
                        Char::ch('\'') => {
                            emit_error!("missing-whitespace-after-doctype-public-keyword");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut public_identifier, .. } = token {
                                *public_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPEPublicIdentifierSingleQuoted);
                        }
                        Char::ch('>') => {
                            emit_error!("missing-doctype-public-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
                                *force_quirks = true;
                            }
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
                                *force_quirks = true;
                            }
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            emit_error!("missing-quote-before-doctype-public-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
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
                            if let Token::DOCTYPE { ref mut public_identifier, .. } = token {
                                *public_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPEPublicIdentifierDoubleQuoted);
                        }
                        Char::ch('\'') => {
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut public_identifier, .. } = token {
                                *public_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPEPublicIdentifierSingleQuoted);
                        }
                        Char::ch('>') => {
                            emit_error!("missing-doctype-public-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
                                *force_quirks = true;
                            }
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
                                *force_quirks = true;
                            }
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            emit_error!("missing-quote-before-doctype-public-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
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
                            self.append_character_to_doctype_public_identifier(REPLACEMENT_CHARACTER);
                        }
                        Char::ch('>') => {
                            emit_error!("abrupt-doctype-public-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
                                *force_quirks = true;
                            }
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
                                *force_quirks = true;
                            }
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            self.append_character_to_doctype_public_identifier(self.current_character);
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
                            self.append_character_to_doctype_public_identifier(REPLACEMENT_CHARACTER);
                        }
                        Char::ch('>') => {
                            emit_error!("abrupt-doctype-public-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
                                *force_quirks = true;
                            }
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
                                *force_quirks = true;
                            }
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            self.append_character_to_doctype_public_identifier(self.current_character);
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
                            emit_error!("missing-whitespace-between-doctype-public-and-system-identifiers");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut system_identifier, .. } = token {
                                *system_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPESytemIdentifierDoubleQuoted);
                        }
                        Char::ch('\'') => {
                            emit_error!("missing-whitespace-between-doctype-public-and-system-identifiers");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut system_identifier, .. } = token {
                                *system_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPESytemIdentifierSingleQuoted);
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
                                *force_quirks = true;
                            }
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            emit_error!("missing-quote-before-doctype-system-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
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
                            if let Token::DOCTYPE { ref mut system_identifier, .. } = token {
                                *system_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPESytemIdentifierDoubleQuoted);
                        }
                        Char::ch('\'') => {
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut system_identifier, .. } = token {
                                *system_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPESytemIdentifierSingleQuoted);
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
                                *force_quirks = true;
                            }
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            emit_error!("missing-quote-before-doctype-system-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
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
                        },
                        Char::ch('"') => {
                            emit_error!("missing-whitespace-after-doctype-system-keyword");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut system_identifier, .. } = token {
                                *system_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPESytemIdentifierDoubleQuoted);
                        }
                        Char::ch('\'') => {
                            emit_error!("missing-whitespace-after-doctype-system-keyword");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut system_identifier, .. } = token {
                                *system_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPESytemIdentifierSingleQuoted);
                        }
                        Char::ch('>') => {
                            emit_error!("missing-doctype-system-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
                                *force_quirks = true;
                            }
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
                                *force_quirks = true;
                            }
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            emit_error!("missing-quote-before-doctype-system-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
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
                            if let Token::DOCTYPE { ref mut system_identifier, .. } = token {
                                *system_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPESytemIdentifierDoubleQuoted);
                        }
                        Char::ch('\'') => {
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut system_identifier, .. } = token {
                                *system_identifier = Some(String::new());
                            }
                            self.switch_to(State::DOCTYPESytemIdentifierSingleQuoted);
                        }
                        Char::ch('>') => {
                            emit_error!("missing-doctype-system-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
                                *force_quirks = true;
                            }
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
                                *force_quirks = true;
                            }
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            emit_error!("missing-quote-before-doctype-system-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
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
                            self.append_character_to_doctype_system_identifier(REPLACEMENT_CHARACTER);
                        }
                        Char::ch('>') => {
                            emit_error!("abrupt-doctype-system-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
                                *force_quirks = true;
                            }
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
                                *force_quirks = true;
                            }
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            self.append_character_to_doctype_system_identifier(self.current_character);
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
                            self.append_character_to_doctype_system_identifier(REPLACEMENT_CHARACTER);
                        }
                        Char::ch('>') => {
                            emit_error!("abrupt-doctype-system-identifier");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
                                *force_quirks = true;
                            }
                            self.switch_to(State::Data);
                            return self.emit_current_token();
                        }
                        Char::eof => {
                            emit_error!("eof-in-doctype");
                            let token = self.current_token.as_mut().unwrap();
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
                                *force_quirks = true;
                            }
                            self.will_emit(self.current_token.clone().unwrap());
                            return self.emit_eof();
                        }
                        _ => {
                            self.append_character_to_doctype_system_identifier(self.current_character);
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
                            if let Token::DOCTYPE { ref mut force_quirks, .. } = token {
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
                State::NamedCharacterReference => {}
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
                State::NumericCharacterReference => {}
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
                State::HexadecimalCharacterReference => {}
                State::DecimalCharacterReference => {}
                State::NumericCharacterReferenceEnd => {}
            }
        }
    }

    fn reconsume_in_return_state(&mut self) {
        self.reconsume_in(self.return_state.clone().unwrap());
    }

    fn new_attribute(&mut self, attribute: Attribute) {
        let token = self.current_token.as_mut().unwrap();
        if let Token::Tag { ref mut attributes, .. } = token {
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
                _ => false
            }
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
        if let Token::DOCTYPE { ref mut public_identifier, .. } = token {
            let public_identifier = public_identifier.as_mut().unwrap();
            public_identifier.push(ch);
        }
    }

    fn append_character_to_doctype_system_identifier(&mut self, ch: char) {
        let token = self.current_token.as_mut().unwrap();
        if let Token::DOCTYPE { ref mut system_identifier, .. } = token {
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
        if let Token::Tag { ref mut attributes, .. } = current_tag {
            let attribute = attributes.last_mut().unwrap();
            attribute.name.push(ch);
        }
    }

    fn append_character_to_attribute_value(&mut self, ch: char) {
        let current_tag = self.current_token.as_mut().unwrap();
        if let Token::Tag { ref mut attributes, .. } = current_tag {
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
        if let Token::Tag { is_end_tag, .. } = &token {
            if !is_end_tag {
                self.last_emitted_start_tag = Some(token.clone());
            }
            // TODO: filter duplicate attributes
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

    fn switch_to(&mut self, state: State) {
        println!("Switch to: {:#?}", state);
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
                    _ => Char::ch(c)
                }
            }
            None => Char::eof
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_comment() {
        let html = "<!--xin chao-->";
        let mut chars = html.chars();
        let mut tokenizer = Tokenizer::new(&mut chars);
        assert_eq!(Token::Comment("xin chao".to_owned()), tokenizer.next_token());
    }

    #[test]
    fn parse_tag() {
        let html = "<html>";
        let mut chars = html.chars();
        let mut tokenizer = Tokenizer::new(&mut chars);
        assert_eq!(Token::Tag {
            tag_name: "html".to_owned(),
            self_closing: false,
            attributes: Vec::new(),
            is_end_tag: false
        }, tokenizer.next_token());
    }

    #[test]
    fn parse_doctype() {
        let html = "<!DOCTYPE html>";
        let mut chars = html.chars();
        let mut tokenizer = Tokenizer::new(&mut chars);
        assert_eq!(Token::DOCTYPE {
            name: Some("html".to_owned()),
            force_quirks: false,
            public_identifier: None,
            system_identifier: None
        }, tokenizer.next_token());
    }

    #[test]
    fn parse_doctype_with_identifiers() {
        let html = r#"<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01 Transitional//EN" "http://www.w3.org/TR/html4/loose.dtd">"#;
        let mut chars = html.chars();
        let mut tokenizer = Tokenizer::new(&mut chars);
        assert_eq!(Token::DOCTYPE {
            name: Some("html".to_owned()),
            force_quirks: false,
            public_identifier: Some("-//W3C//DTD HTML 4.01 Transitional//EN".to_owned()),
            system_identifier: Some("http://www.w3.org/TR/html4/loose.dtd".to_owned())
        }, tokenizer.next_token());
    }
}
