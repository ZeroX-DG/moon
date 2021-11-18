use crate::property::Property;
use crate::value::Value;
use crate::value_processing::ComputeContext;
use crate::value_processing::ValueRef;
use crate::values::length::Length;
use crate::values::length::LengthUnit;

const BASE_FONT_SIZE: f32 = 12.; // 12px

pub fn compute_font_size(value: &Value, context: &mut ComputeContext) -> ValueRef {
    match value {
        Value::Percentage(percentage) => {
            let parent_font_size = context
                .parent
                .as_ref()
                .map(|parent| {
                    if let Some(p) = parent.upgrade() {
                        return Some(p.borrow().get_style(&Property::FontSize).to_absolute_px());
                    }
                    None
                })
                .unwrap_or_default();
            let value = parent_font_size
                .map(|font_size| Value::Length(Length::new_px(percentage.to_px(font_size))))
                // TODO: investigate what will happen if we set percentage in font-size for
                // HTML element
                .unwrap_or(Value::Length(Length::new_px(
                    percentage.to_px(BASE_FONT_SIZE),
                )));
            context.style_cache.get(&value)
        }
        Value::Length(Length {
            value: _,
            unit: LengthUnit::Px,
        }) => context.style_cache.get(value),
        Value::Length(Length {
            value,
            unit: LengthUnit::Em,
        }) => {
            let parent_font_size = context
                .parent
                .as_ref()
                .map(|parent| {
                    if let Some(p) = parent.upgrade() {
                        return Some(p.borrow().get_style(&Property::FontSize).to_absolute_px());
                    }
                    None
                })
                .unwrap_or_default();
            let value = parent_font_size
                .map(|font_size| Value::Length(Length::new_px(value.0 * font_size)))
                // TODO: investigate what will happen if we set percentage in font-size for
                // HTML element
                .unwrap_or(Value::Length(Length::new_px(value.0 * BASE_FONT_SIZE)));
            context.style_cache.get(&value)
        }
        _ => unreachable!("Invalid value provided for font-size"),
    }
}
