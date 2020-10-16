use crate::style_tree::Value;
use css::tokenizer::token::Token;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Color {
    RGBA(u8, u8, u8, u8),
}

impl Color {
    pub fn parse(tokens: Vec<Token>) -> Option<Value> {
        Some(Value::Color(Color::RGBA(255, 255, 255, 255)))
    }

    pub fn default() -> Value {
        Value::Color(Color::RGBA(0, 0, 0, 0))
    }
}
