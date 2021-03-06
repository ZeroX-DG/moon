use super::prelude::{Length, LengthPercentage};
use css::parser::structs::ComponentValue;
use css::tokenizer::token::Token;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct BorderRadius(pub LengthPercentage, pub LengthPercentage);

impl BorderRadius {
    pub fn parse(values: &[ComponentValue]) -> Option<Self> {
        let mut data = Vec::new();
        for value in values {
            match value {
                ComponentValue::PerservedToken(Token::Dimension { .. })
                | ComponentValue::PerservedToken(Token::Percentage(..)) => data.push(value),
                _ => {}
            }
        }

        let values_count = data.len();

        if values_count == 1 {
            let p = match LengthPercentage::parse(values) {
                Some(p) => p,
                _ => return None,
            };
            return Some(Self(p.clone(), p.clone()));
        }

        if values_count == 2 {
            let horizontal_r = match LengthPercentage::parse(&[data[0].clone()]) {
                Some(p) => p,
                _ => return None,
            };

            let vertical_r = match LengthPercentage::parse(&[data[1].clone()]) {
                Some(p) => p,
                _ => return None,
            };

            return Some(Self(horizontal_r, vertical_r));
        }

        None
    }

    pub fn zero() -> Self {
        Self(
            LengthPercentage::Length(Length::zero()),
            LengthPercentage::Length(Length::zero()),
        )
    }
}
