use css::{parser::structs::ComponentValue, tokenizer::token::Token};

use super::number::Number;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct FontWeight(pub Number);

impl Eq for FontWeight {}

impl FontWeight {
    pub fn parse(values: &[ComponentValue]) -> Option<Self> {
        match values.iter().next() {
            Some(ComponentValue::PerservedToken(Token::Ident(value))) => {
                if value.eq_ignore_ascii_case("normal") {
                    Some(FontWeight(Number(400.)))
                } else if value.eq_ignore_ascii_case("bold") {
                    Some(FontWeight(Number(700.)))
                } else {
                    None
                }
            }
            Some(ComponentValue::PerservedToken(Token::Number { value, .. })) => {
                if *value >= 100. && *value <= 900. && *value % 100. == 0. {
                    Some(FontWeight(Number(*value)))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn value(&self) -> f32 {
        self.0 .0
    }
}
