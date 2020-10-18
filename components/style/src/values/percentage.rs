use css::parser::structs::ComponentValue;
use css::tokenizer::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub struct Percentage(pub f32);

impl Eq for Percentage {}

impl Percentage {
    pub fn parse(values: &Vec<ComponentValue>) -> Option<Self> {
        match values.first() {
            Some(ComponentValue::PerservedToken(Token::Percentage(value))) => {
                Some(Percentage(*value))
            }
            _ => None,
        }
    }
}


