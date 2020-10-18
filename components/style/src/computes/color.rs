use crate::value_processing::{compute, ComputeContext};
use crate::value_processing::{Property, Value};
use crate::values::color::Color;

pub fn compute_color(value: &Value, property: &Property, context: &ComputeContext) -> Value {
    match value {
        Value::Color(Color::CurrentColor) => match property {
            Property::Color => {
                if let Some(parent) = &context.parent {
                    if let Some(p) = parent.upgrade() {
                        return p.borrow().get_style(&property);
                    }
                }
                Value::initial(property)
            }
            _ => {
                // It's guarentee that all properties have a vlue
                compute(
                    &Property::Color,
                    context.properties.get(&Property::Color).unwrap(),
                    context,
                )
            }
        },
        _ => value.clone(),
    }
}
