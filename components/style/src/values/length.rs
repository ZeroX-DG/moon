use super::number::Number;
use css::parser::structs::ComponentValue;
use css::tokenizer::token::Token;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Length {
    pub value: Number,
    pub unit: LengthUnit,
}

impl Eq for Length {}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum LengthUnit {
    Em,
    Ex,
    In,
    Cm,
    Mm,
    Pt,
    Pc,
    Px,
}

impl LengthUnit {
    pub fn from_str(unit: &str) -> Option<Self> {
        match unit {
            "em" => Some(LengthUnit::Em),
            "ex" => Some(LengthUnit::Ex),
            "in" => Some(LengthUnit::In),
            "cm" => Some(LengthUnit::Cm),
            "mm" => Some(LengthUnit::Mm),
            "pt" => Some(LengthUnit::Pt),
            "pc" => Some(LengthUnit::Pc),
            "px" => Some(LengthUnit::Px),
            _ => None,
        }
    }
}

impl Length {
    pub fn parse(values: &[ComponentValue]) -> Option<Self> {
        match values.first() {
            Some(ComponentValue::PerservedToken(Token::Dimension { value, unit, .. })) => {
                if let Some(unit) = LengthUnit::from_str(&unit) {
                    return Some(Length {
                        value: (*value).into(),
                        unit,
                    });
                }
                None
            }
            Some(ComponentValue::PerservedToken(Token::Number { value, .. })) => {
                if *value == 0.0 {
                    return Some(Length::zero());
                }
                None
            }
            _ => None,
        }
    }

    pub fn zero() -> Self {
        Length {
            value: 0.0.into(),
            unit: LengthUnit::Px,
        }
    }
}
