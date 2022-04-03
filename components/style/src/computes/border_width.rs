use crate::property::Property;
use crate::value::Value;
use crate::value_processing::ComputeContext;
use crate::value_processing::ValueRef;
use crate::values::border_style::BorderStyle;
use crate::values::length::Length;

pub fn compute_border_width(property: &Property, value: &Value, context: &mut ComputeContext) -> ValueRef {
    let border_style = match &property {
        Property::BorderTopWidth => Property::BorderTopStyle,
        Property::BorderLeftWidth => Property::BorderLeftStyle,
        Property::BorderBottomWidth => Property::BorderBottomStyle,
        Property::BorderRightWidth => Property::BorderRightStyle,
        _ => unreachable!()
    };
    if let Some(border_style) = context.properties.get(&border_style) {
        return match border_style {
            Value::BorderStyle(BorderStyle::None)
            | Value::BorderStyle(BorderStyle::Hidden) => {
                let value = Value::Length(Length::zero());
                context.style_cache.get(&value)
            }
            _ => context.style_cache.get(value),
        }
    }
    return context.style_cache.get(value);
}
