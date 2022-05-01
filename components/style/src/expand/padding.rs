use super::ExpandOutput;
use css::parser::structs::ComponentValue;
use style_types::{Property, Value};

pub fn expand_padding(values: &[&[ComponentValue]]) -> ExpandOutput {
    if values.len() == 1 {
        // this is a single value
        let value = Value::parse(&Property::PaddingTop, values[0]);

        if value.is_none() {
            return None;
        }

        return Some(vec![
            (Property::PaddingTop, value.clone()),
            (Property::PaddingRight, value.clone()),
            (Property::PaddingBottom, value.clone()),
            (Property::PaddingLeft, value),
        ]);
    }

    if values.len() == 2 {
        let padding_y = Value::parse(&Property::PaddingLeft, values[0]);
        let padding_x = Value::parse(&Property::PaddingLeft, values[1]);

        if padding_x.is_none() || padding_y.is_none() {
            return None;
        }

        return Some(vec![
            (Property::PaddingTop, padding_y.clone()),
            (Property::PaddingRight, padding_x.clone()),
            (Property::PaddingBottom, padding_y),
            (Property::PaddingLeft, padding_x),
        ]);
    }

    if values.len() <= 4 {
        let padding_top = Value::parse(&Property::PaddingRight, values[0]);
        let padding_right = Value::parse(&Property::PaddingRight, values[1]);
        let padding_bottom = Value::parse(&Property::PaddingRight, values[2]);

        if padding_top.is_none() || padding_right.is_none() || padding_bottom.is_none() {
            return None;
        }

        if values.len() == 3 {
            return Some(vec![
                (Property::PaddingTop, padding_top),
                (Property::PaddingRight, padding_right),
                (Property::PaddingBottom, padding_bottom),
                (Property::PaddingLeft, None),
            ]);
        }

        let padding_left = Value::parse(&Property::PaddingRight, values[3]);

        if padding_left.is_none() {
            return None;
        }

        return Some(vec![
            (Property::PaddingTop, padding_top),
            (Property::PaddingRight, padding_right),
            (Property::PaddingBottom, padding_bottom),
            (Property::PaddingLeft, padding_left),
        ]);
    }

    None
}
