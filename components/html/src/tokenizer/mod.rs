mod state;
mod token;

use std::collections::{VecDeque};
use std::str::Chars;
use std::env;
use state::State;
use token::Token;


fn is_trace() -> bool { match env::var("TRACE_TOKENIZER") {
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
    last_emitted_start_tag: Option<Token>
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
            last_emitted_start_tag: None
        }
    }

    pub fn next_token(&mut self) -> Token {
        if !self.output.is_empty() {
            return self.output.pop_front().unwrap();
        }
        loop {
            let ch = self.consume_next();
            match self.state {
                State::Data => {
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
                    match ch {
                        Char::ch('&') => {
                            self.return_state = Some(State::RCDATA);
                            self.switch_to(State::CharacterReference);
                        }
                        Char::ch('<') => self.switch_to(State::RCDATALessThanSign),
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            // TODO: replace with char::REPLACEMENT_CHARACTER when stable
                            return self.emit_char('\u{FFFD}');
                        }
                        Char::eof => return self.emit_eof(),
                        _ => return self.emit_current_char()
                    }
                }
                State::RAWTEXT => {
                    match ch {
                        Char::ch('<') => self.switch_to(State::RAWTEXTLessThanSign),
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            // TODO: replace with char::REPLACEMENT_CHARACTER when stable
                            return self.emit_char('\u{FFFD}');
                        }
                        Char::eof => return self.emit_eof(),
                        _ => return self.emit_current_char()
                    }
                }
                State::ScriptData => {
                    match ch {
                        Char::ch('<') => self.switch_to(State::ScriptDataLessThanSign),
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            // TODO: replace with char::REPLACEMENT_CHARACTER when stable
                            self.emit_char('\u{FFFD}');
                        }
                        Char::eof => return self.emit_eof(),
                        _ => return self.emit_current_char()
                    }
                }
                State::PLAINTEXT => {
                    match ch {
                        Char::null => {
                            emit_error!("unexpected-null-character");
                            // TODO: replace with char::REPLACEMENT_CHARACTER when stable
                            return self.emit_char('\u{FFFD}');
                        }
                        Char::eof => return self.emit_eof(),
                        Char::ch(_) => return self.emit_current_char()
                    }
                }
                State::TagOpen => {
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
                            // TODO: replace with char::REPLACEMENT_CHARACTER when stable
                            self.append_character_to_tag_name('\u{FFFD}');
                        } Char::eof => { emit_error!("eof-in-tag");
                            return self.emit_eof();
                        }
                        _ => {
                            self.append_character_to_tag_name(self.current_character);
                        }
                    }
                }
                State::RCDATALessThanSign => {
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
                            self.append_character_to_tag_name(self.current_character.to_ascii_lowercase());
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
                            self.append_character_to_tag_name(self.current_character.to_ascii_lowercase());
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
                            self.append_character_to_tag_name(self.current_character.to_ascii_lowercase());
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
                            // TODO: replace with char::REPLACEMENT_CHARACTER when stable
                            return self.emit_char('\u{FFFD}');
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
                            // TODO: replace with char::REPLACEMENT_CHARACTER when stable
                            return self.emit_char('\u{FFFD}');
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
                            // TODO: replace with char::REPLACEMENT_CHARACTER when stable
                            return self.emit_char('\u{FFFD}');
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
                            return self.emit_char('\u{FFFD}');
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
                            return self.emit_char('\u{FFFD}');
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
                            return self.emit_char('\u{FFFD}');
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
                State::BeforeAttributeName => {}
                State::AttributeName => {}
                State::AfterAttributeName => {}
                State::BeforeAttributeValue => {}
                State::AttributeValueDoubleQuoted => {}
                State::AttributeValueSingleQuoted => {}
                State::AttributeValueUnQuoted => {}
                State::AfterAttributeValueQuoted => {}
                State::SelfClosingStartTag => {
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
                            self.append_character_to_token_data('\u{FFFD}');
                        }
                        _ => {
                            self.append_character_to_token_data(self.current_character);
                        }
                    }
                }
                State::MarkupDeclarationOpen => {}
                State::CommentStart => {
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
                            self.append_character_to_token_data('\u{FFFD}');
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
                State::CommentEnd => {}
                State::CommentEndBang => {}
                State::DOCTYPE => {}
                State::BeforeDOCTYPEName => {}
                State::DOCTYPEName => {}
                State::AfterDOCTYPEName => {}
                State::AfterDOCTYPEPublicKeyword => {}
                State::BeforeDOCTYPEPublicIdentifier => {}
                State::DOCTYPEPublicIdentifierDoubleQuoted => {}
                State::DOCTYPEPublicIdentifierSingleQuoted => {}
                State::AfterDOCTYPEPublicIdentifier => {}
                State::BetweenDOCTYPEPublicAndSystemIdentifiers => {}
                State::AfterDOCTYPESystemKeyword => {}
                State::BeforeDOCTYPESystemIdentifier => {}
                State::DOCTYPESytemIdentifierDoubleQuoted => {}
                State::DOCTYPESytemIdentifierSingleQuoted => {}
                State::AfterDOCTYPESystemIdentifier => {}
                State::BogusDOCTYPE => {}
                State::CDATASection => {
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
                State::AmbiguousAmpersand => {}
                State::NumericCharacterReference => {}
                State::HexadecimalCharacterReferenceStart => {}
                State::DecimalCharacterReferenceStart => {}
                State::HexadecimalCharacterReference => {}
                State::DecimalCharacterReference => {}
                State::NumericCharacterReferenceEnd => {}
            }
        }
    }

    fn reconsume_in_return_state(&mut self) {
        self.reconsume_in(self.return_state.clone().unwrap());
    }

    fn flush_code_points_consumed_as_a_character_reference(&mut self) {
        if self.is_character_part_of_attribute() {
            for c in self.temp_buffer.chars() {
                // TODO: implement attribute value
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
        self.state = state;
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

