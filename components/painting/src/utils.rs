use shared::{color::Color, primitive::Radii};
use style_types::{
    values::prelude::{BorderRadius, LengthPercentage},
    Value,
};

pub fn is_zero(value: &Value) -> bool {
    match value {
        Value::Length(l) => *l.value == 0.0,
        Value::Percentage(p) => *p.0 == 0.0,
        _ => false,
    }
}

pub fn color_from_value(color: &Value) -> Color {
    match color {
        Value::Color(c) => c.into(),
        _ => Color {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        },
    }
}

pub fn to_radii(value: &Value, width: f32, font_size: f32) -> Radii {
    let (h, v) = match value {
        Value::BorderRadius(BorderRadius(hr, vr)) => (
            resolve_length_percentage(hr, width, font_size),
            resolve_length_percentage(vr, width, font_size),
        ),
        _ => (0., 0.),
    };

    Radii::new(h, v)
}

fn resolve_length_percentage(value: &LengthPercentage, width: f32, font_size: f32) -> f32 {
    match value {
        LengthPercentage::Length(l) => l.resolve(font_size),
        LengthPercentage::Percentage(p) => p.to_px(width),
    }
}
