use css::parser::structs::ComponentValue;
use css::tokenizer::token::Token;

use crate::property::Property;
use crate::value::Value;
use crate::value_processing::{ComputeContext, ValueRef};

use super::border_style::BorderStyle;
use super::length::Length;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum BorderWidth {
    Thin,
    Medium,
    Thick,
}

impl BorderWidth {
    pub fn parse(values: &[ComponentValue]) -> Option<Self> {
        match values.iter().next() {
            Some(ComponentValue::PerservedToken(Token::Ident(value))) => match value {
                v if v.eq_ignore_ascii_case("thin") => Some(BorderWidth::Thin),
                v if v.eq_ignore_ascii_case("medium") => Some(BorderWidth::Medium),
                v if v.eq_ignore_ascii_case("thick") => Some(BorderWidth::Thick),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn to_px(&self) -> f32 {
        match &self {
            BorderWidth::Thin => 1.,
            BorderWidth::Medium => 3.,
            BorderWidth::Thick => 5.,
        }
    }

    pub fn compute(&self, property: &Property, context: &mut ComputeContext) -> ValueRef {
        let border_style = match &property {
            Property::BorderTopWidth => Property::BorderTopStyle,
            Property::BorderLeftWidth => Property::BorderLeftStyle,
            Property::BorderBottomWidth => Property::BorderBottomStyle,
            Property::BorderRightWidth => Property::BorderRightStyle,
            _ => unreachable!(),
        };
        if let Some(border_style) = context.properties.get(&border_style) {
            return match border_style {
                Value::BorderStyle(BorderStyle::None) | Value::BorderStyle(BorderStyle::Hidden) => {
                    let value = Value::Length(Length::zero());
                    context.style_cache.get(&value)
                }
                _ => context.style_cache.get(&Value::BorderWidth(self.clone())),
            };
        }
        return context.style_cache.get(&Value::BorderWidth(self.clone()));
    }
}
