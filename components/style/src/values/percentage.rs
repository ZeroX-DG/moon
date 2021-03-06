use super::number::Number;
use css::parser::structs::ComponentValue;
use css::tokenizer::token::Token;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Percentage(pub Number);

impl Eq for Percentage {}

impl Percentage {
    pub fn parse(values: &[ComponentValue]) -> Option<Self> {
        match values.first() {
            Some(ComponentValue::PerservedToken(Token::Percentage(value))) => {
                Some(Percentage((*value).into()))
            }
            _ => None,
        }
    }

    pub fn to_px(&self, containing: f32) -> f32 {
        *self.0 * containing / 100.0
    }
}
