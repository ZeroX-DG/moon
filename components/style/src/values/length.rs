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
    Rem,
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
            "rem" => Some(LengthUnit::Rem),
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
    pub fn new(value: f32, unit: LengthUnit) -> Self {
        Self {
            value: value.into(),
            unit,
        }
    }

    pub fn new_px(value: f32) -> Self {
        Self::new(value, LengthUnit::Px)
    }

    pub fn zero() -> Self {
        Length {
            value: 0.0.into(),
            unit: LengthUnit::Px,
        }
    }

    pub fn to_px(&self) -> f32 {
        match self.unit {
            LengthUnit::Px => *self.value,
            _ => unreachable!("Calling to_px on non-px length unit"),
        }
    }

    pub fn resolve(&self, font_size: f32) -> f32 {
        match self.unit {
            LengthUnit::Px => *self.value,
            LengthUnit::Em | LengthUnit::Rem => *self.value * font_size,
            _ => unimplemented!("Calling resolve on unsupported length unit"),
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
}
