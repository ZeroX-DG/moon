use crate::property::Property;
use crate::render_tree::RenderNodePtr;
use crate::value::Value;
use crate::value_processing::ComputeContext;
use crate::value_processing::ValueRef;
use crate::values::length::Length;
use crate::values::length::LengthUnit;

const BASE_FONT_SIZE: f32 = 16.; // 16px

pub fn compute_padding(value: &Value, context: &mut ComputeContext) -> ValueRef {
    match value {
        Value::Length(Length {
            value,
            unit: LengthUnit::Em,
        }) => {
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
            let value = Value::Length(Length::new_px(value.0 * parent_font_size));
            context.style_cache.get(&value)
        }
        _ => context.style_cache.get(value),
    }
}
