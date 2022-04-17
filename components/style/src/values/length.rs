use crate::property::Property;
use crate::render_tree::RenderNodePtr;
use crate::value::Value;
use crate::value_processing::{ComputeContext, ValueRef};

use super::number::Number;
use css::parser::structs::ComponentValue;
use css::tokenizer::token::Token;

const BASE_FONT_SIZE: f32 = 16.; // 16px

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

    pub fn compute_value(&self, _: &Property, context: &mut ComputeContext) -> Self {
        match self {
            Length {
                value,
                unit: LengthUnit::Em,
            } => {
                let parent_font_size = context
                    .parent
                    .as_ref()
                    .map(|parent| {
                        if let Some(p) = parent.upgrade() {
                            return RenderNodePtr(p)
                                .get_style(&Property::FontSize)
                                .to_absolute_px();
                        }
                        BASE_FONT_SIZE
                    })
                    .unwrap_or_default();
                Length::new_px(value.0 * parent_font_size)
            }
            Length {
                value,
                unit: LengthUnit::Rem,
            } => {
                let root_font_size = context
                    .root
                    .as_ref()
                    .map(|root| {
                        if let Some(root) = root.upgrade() {
                            return RenderNodePtr(root)
                                .get_style(&Property::FontSize)
                                .to_absolute_px();
                        }
                        BASE_FONT_SIZE
                    })
                    .unwrap_or(BASE_FONT_SIZE);
                Length::new_px(value.0 * root_font_size)
            }
            _ => self.clone(),
        }
    }

    pub fn compute(&self, property: &Property, context: &mut ComputeContext) -> ValueRef {
        let value = self.compute_value(property, context);
        context.style_cache.get(&Value::Length(value))
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
