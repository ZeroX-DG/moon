use crate::property::Property;
use crate::value::Value;
use crate::value_processing::ValueRef;
use crate::value_processing::ComputeContext;
use crate::values::length::Length;
use crate::values::length::LengthUnit;

pub fn compute_length(value: &Value, property: &Property, context: &mut ComputeContext) -> ValueRef {
    match value {
        Value::Length(Length { value: _, unit: LengthUnit::Px }) => {
            if !context.style_cache.contains(value) {
                context.style_cache.insert(ValueRef::new(value.clone()));
            }
            context.style_cache.get(value).unwrap().clone()
        },
        Value::Length(Length { value, unit: LengthUnit::Em }) => match property {
            Property::FontSize => {
                let parent_font_size = context.parent.as_ref().map(|parent| {
                    if let Some(p) = parent.upgrade() {
                        return Some(p.borrow().get_style(&property).to_absolute_px());
                    }
                    None
                }).unwrap_or_default();
                let value = parent_font_size
                    .map(|font_size| Value::Length(Length::new_px(value.0 * font_size)))
                    // TODO: investigate what will happen if we set percentage in font-size for
                    // HTML element
                    .unwrap_or(Value::Length(Length::new_px(value.0 * 12.)));
                if !context.style_cache.contains(&value) {
                    context.style_cache.insert(ValueRef::new(value.clone()));
                }
                context.style_cache.get(&value).unwrap().clone()
            }
            _ => unimplemented!()
        },
        _ => unreachable!("Cannot compute length for non-length value")
    }
}
