use css::tokenizer::token::Token;
use crate::style_tree::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Display {
    Inline,
    Block,
    None
}

impl Display {
    pub fn parse(tokens: Vec<Token>) -> Option<Value> {
        Some(Value::Display(Display::Block))
    }

    pub fn default() -> Value {
        Value::Display(Display::Block)
    }
}
