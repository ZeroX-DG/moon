use super::number::Number;
use css::parser::structs::ComponentValue;
use css::tokenizer::token::Token;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Percentage(pub Number);

impl Eq for Percentage {}

impl Percentage {
    pub fn parse(values: &Vec<ComponentValue>) -> Option<Self> {
        match values.first() {
            Some(ComponentValue::PerservedToken(Token::Percentage(value))) => {
                Some(Percentage((*value).into()))
            }
            _ => None,
        }
    }
}
