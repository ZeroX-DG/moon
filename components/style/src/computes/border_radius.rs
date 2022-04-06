use crate::property::Property;
use crate::render_tree::RenderNodePtr;
use crate::value::Value;
use crate::value_processing::ComputeContext;
use crate::value_processing::ValueRef;
use crate::values::border_radius::BorderRadius;
use crate::values::length::Length;
use crate::values::length::LengthUnit;
use crate::values::length_percentage::LengthPercentage;

const BASE_FONT_SIZE: f32 = 16.; // 16px

pub fn compute_border_radius(value: &Value, context: &mut ComputeContext) -> ValueRef {
    match value {
        Value::BorderRadius(BorderRadius(hr, vr)) => {
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

            let hr_value = if let LengthPercentage::Length(Length {
                value,
                unit: LengthUnit::Em,
            }) = hr
            {
                LengthPercentage::Length(Length::new_px(value.0 * parent_font_size))
            } else {
                hr.clone()
            };

            let vr_value = if let LengthPercentage::Length(Length {
                value,
                unit: LengthUnit::Em,
            }) = vr
            {
                LengthPercentage::Length(Length::new_px(value.0 * parent_font_size))
            } else {
                vr.clone()
            };

            let value = Value::BorderRadius(BorderRadius(hr_value, vr_value));
            context.style_cache.get(&value)
        }
        _ => context.style_cache.get(value),
    }
}
