use crate::property::Property;
use crate::render_tree::RenderNodePtr;
use crate::value::Value;
use crate::value_processing::{ComputeContext, ValueRef};

use super::length::Length;
use super::number::Number;
use css::parser::structs::ComponentValue;
use css::tokenizer::token::Token;

const BASE_FONT_SIZE: f32 = 16.; // 16px

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

    pub fn compute(&self, property: &Property, context: &mut ComputeContext) -> ValueRef {
        if let Property::FontSize = property {
            log::debug!("HHHHHHHHHHHHHHHHHHHHH");
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
                .unwrap_or(BASE_FONT_SIZE);
            let value = Value::Length(Length::new_px(self.to_px(parent_font_size)));
            return context.style_cache.get(&value);
        }
        context.style_cache.get(&Value::Percentage(self.clone()))
    }
}
