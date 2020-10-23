use super::ExpandOutput;
use crate::value_processing::{Property, Value};
use css::parser::structs::ComponentValue;

pub fn expand_border_width(values: &[&[ComponentValue]]) -> ExpandOutput {
    if values.len() == 1 {
        let value = Value::parse(&Property::BorderTopWidth, values[0]);

        if value.is_none() {
            return None;
        }

        return Some(vec![
            (Property::BorderTopWidth, value.clone()),
            (Property::BorderRightWidth, value.clone()),
            (Property::BorderBottomWidth, value.clone()),
            (Property::BorderLeftWidth, value),
        ]);
    }

    if values.len() == 2 {
        let border_y = Value::parse(&Property::BorderTopWidth, values[0]);
        let border_x = Value::parse(&Property::BorderTopWidth, values[1]);

        if border_x.is_none() || border_y.is_none() {
            return None;
        }

        return Some(vec![
            (Property::BorderTopWidth, border_y.clone()),
            (Property::BorderRightWidth, border_x.clone()),
            (Property::BorderBottomWidth, border_y),
            (Property::BorderLeftWidth, border_x),
        ]);
    }

    if values.len() <= 4 {
        let border_top = Value::parse(&Property::BorderTopWidth, values[0]);
        let border_right = Value::parse(&Property::BorderTopWidth, values[1]);
        let border_bottom = Value::parse(&Property::BorderTopWidth, values[2]);

        if border_top.is_none() || border_right.is_none() || border_bottom.is_none() {
            return None;
        }

        if values.len() == 3 {
            return Some(vec![
                (Property::BorderTopWidth, border_top),
                (Property::BorderRightWidth, border_right),
                (Property::BorderBottomWidth, border_bottom),
                (Property::BorderLeftWidth, None),
            ]);
        }

        let border_left = Value::parse(&Property::BorderTopWidth, values[3]);

        if border_left.is_none() {
            return None;
        }

        return Some(vec![
            (Property::BorderTopWidth, border_top),
            (Property::BorderRightWidth, border_right),
            (Property::BorderBottomWidth, border_bottom),
            (Property::BorderLeftWidth, border_left),
        ]);
    }

    None
}
