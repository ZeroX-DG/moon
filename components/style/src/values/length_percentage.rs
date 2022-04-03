use css::parser::structs::ComponentValue;

use super::length::Length;
use super::percentage::Percentage;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum LengthPercentage {
    Length(Length),
    Percentage(Percentage),
}

impl LengthPercentage {
    pub fn is_zero(&self) -> bool {
        match self {
            LengthPercentage::Length(l) => *l.value == 0.0,
            LengthPercentage::Percentage(p) => *p.0 == 0.0,
        }
    }

    pub fn to_px(&self, relative_to: f32) -> f32 {
        match self {
            LengthPercentage::Length(l) => l.to_px(relative_to),
            LengthPercentage::Percentage(p) => p.to_px(relative_to),
        }
    }

    pub fn parse(values: &[ComponentValue]) -> Option<Self> {
        match Length::parse(values) {
            Some(l) => Some(Self::Length(l)),
            None => match Percentage::parse(values) {
                Some(p) => Some(Self::Percentage(p)),
                None => None,
            },
        }
    }
}
