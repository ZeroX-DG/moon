use style::value::Value;
use shared::color::Color;

pub fn is_zero(value: &Value) -> bool {
    match value {
        Value::Length(l) => l.to_px() == 0.0,
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
            a: 0
        }
    }
}
