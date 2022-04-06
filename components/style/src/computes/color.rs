use crate::property::Property;
use crate::render_tree::RenderNodePtr;
use crate::value::Value;
use crate::value_processing::ComputeContext;
use crate::value_processing::ValueRef;
use crate::values::color::Color;

pub fn compute_color(value: &Value, context: &mut ComputeContext) -> ValueRef {
    match value {
        Value::Color(Color::CurrentColor) => {
            if let Some(parent) = &context.parent {
                if let Some(p) = parent.upgrade() {
                    return RenderNodePtr(p).get_style(&Property::Color);
                }
            }
            let value = Value::initial(&Property::Color);
            context.style_cache.get(&value)
        }
        _ => context.style_cache.get(value),
    }
}
