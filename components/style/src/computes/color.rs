use crate::property::Property;
use crate::value::Value;
use crate::value_processing::ValueRef;
use crate::value_processing::ComputeContext;
use crate::values::color::Color;

pub fn compute_color(value: &Value, context: &mut ComputeContext) -> ValueRef {
    match value {
        Value::Color(Color::CurrentColor) => {
            if let Some(parent) = &context.parent {
                if let Some(p) = parent.upgrade() {
                    return p.borrow().get_style(&Property::Color);
                }
            }
            let value = Value::initial(&Property::Color);
            if !context.style_cache.contains(&value) {
                context.style_cache.insert(ValueRef::new(value.clone()));
            }
            context.style_cache.get(&value).unwrap().clone() 
        },
        _ => {
            if !context.style_cache.contains(value) {
                context.style_cache.insert(ValueRef::new(value.clone()));
            }
            context.style_cache.get(value).unwrap().clone()
        }
    }
}
