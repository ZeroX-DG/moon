use super::ExpandOutput;
use crate::value_processing::{Property, Value};
use css::parser::structs::ComponentValue;

pub fn expand_border_radius(values: &[&[ComponentValue]]) -> ExpandOutput {
    if values.len() == 1 {
        // this is a single value
        let value = Value::parse(&Property::BorderTopLeftRadius, values[0]);

        if value.is_none() {
            return None;
        }

        return Some(vec![
            (Property::BorderTopLeftRadius, value.clone()),
            (Property::BorderTopRightRadius, value.clone()),
            (Property::BorderBottomLeftRadius, value.clone()),
            (Property::BorderBottomRightRadius, value),
        ]);
    }

    if values.len() == 2 {
        let top_left_bottom_right = Value::parse(&Property::BorderTopRightRadius, values[0]);
        let top_right_bottom_left = Value::parse(&Property::BorderTopRightRadius, values[1]);

        if top_left_bottom_right.is_none() || top_right_bottom_left.is_none() {
            return None;
        }

        return Some(vec![
            (Property::BorderTopLeftRadius, top_left_bottom_right.clone()),
            (Property::BorderTopRightRadius, top_right_bottom_left.clone()),
            (Property::BorderBottomLeftRadius, top_right_bottom_left),
            (Property::BorderBottomRightRadius, top_left_bottom_right),
        ]);
    }

    if values.len() == 3 {
        let top_left = Value::parse(&Property::BorderTopLeftRadius, values[0]);
        let top_right_bottom_left = Value::parse(&Property::BorderTopRightRadius, values[1]);
        let bottom_right = Value::parse(&Property::BorderBottomRightRadius, values[2]);

        if top_left.is_none() || top_right_bottom_left.is_none() || bottom_right.is_none() {
            return None;
        }

        return Some(vec![
            (Property::BorderTopLeftRadius, top_left),
            (Property::BorderTopRightRadius, top_right_bottom_left.clone()),
            (Property::BorderBottomLeftRadius, top_right_bottom_left),
            (Property::BorderBottomRightRadius, bottom_right),
        ]);
    }

    if values.len() == 4 {
        let top_left = Value::parse(&Property::BorderTopLeftRadius, values[0]);
        let top_right = Value::parse(&Property::BorderTopLeftRadius, values[1]);
        let bottom_right = Value::parse(&Property::BorderTopLeftRadius, values[2]);
        let bottom_left = Value::parse(&Property::BorderTopLeftRadius, values[3]);

        if top_left.is_none() || top_right.is_none() || bottom_right.is_none() || bottom_left.is_none() {
            return None;
        }

        return Some(vec![
            (Property::BorderTopLeftRadius, top_left),
            (Property::BorderTopRightRadius, top_right),
            (Property::BorderBottomLeftRadius, bottom_left),
            (Property::BorderBottomRightRadius, bottom_right),
        ]);
    }

    None
}
