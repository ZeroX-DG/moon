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
            LengthPercentage::Length(l) => l.to_px() == 0.0,
            LengthPercentage::Percentage(p) => *p.0 == 0.0,
        }
    }

    pub fn to_px(&self, containing: f32) -> f32 {
        match self {
            LengthPercentage::Length(l) => l.to_px(),
            LengthPercentage::Percentage(p) => p.to_px(containing),
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
