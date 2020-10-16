use crate::style_tree::Value;
use css::tokenizer::token::Token;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Display {
    Inline,
    Block,
    None,
}

impl Display {
    pub fn parse(tokens: Vec<Token>) -> Option<Value> {
        Some(Value::Display(Display::Block))
    }

    pub fn default() -> Value {
        Value::Display(Display::Block)
    }
}
