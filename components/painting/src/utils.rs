use style::value_processing::Value;

pub fn is_zero(value: &Value) -> bool {
    match value {
        Value::Length(l) => l.to_px() == 0.0,
        Value::Percentage(p) => *p.0 == 0.0,
        _ => false,
    }
}
