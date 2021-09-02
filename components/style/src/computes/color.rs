use crate::property::Property;
use crate::value::Value;
use crate::value_processing::ValueRef;
use crate::value_processing::{compute, ComputeContext};
use crate::values::color::Color;

pub fn compute_color(value: &Value, property: &Property, context: &mut ComputeContext) -> ValueRef {
    match value {
        Value::Color(Color::CurrentColor) => match property {
            Property::Color => {
                if let Some(parent) = &context.parent {
                    if let Some(p) = parent.upgrade() {
                        return p.borrow().get_style(&property);
                    }
                }
                let value = Value::initial(property);
                if !context.style_cache.contains(&value) {
                    context.style_cache.insert(ValueRef::new(value.clone()));
                }
                context.style_cache.get(&value).unwrap().clone()
            }
            _ => {
                // It's guarentee that all properties have a vlue
                let color = context.properties.get(&Property::Color).unwrap().clone();
                compute(&Property::Color, &color, context)
            }
        },
        _ => {
            if !context.style_cache.contains(value) {
                context.style_cache.insert(ValueRef::new(value.clone()));
            }
            context.style_cache.get(value).unwrap().clone()
        }
    }
}
