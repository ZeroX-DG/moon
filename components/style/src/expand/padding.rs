use super::ExpandOutput;
use crate::value_processing::{Property, Value};
use css::parser::structs::ComponentValue;

pub fn expand_padding(values: &[&[ComponentValue]]) -> ExpandOutput {
    if values.len() == 1 {
        // this is a single value
        let value = Value::parse(&Property::MarginTop, values[0]);

        return Some(vec![
            (Property::PaddingTop, value.clone()),
            (Property::PaddingRight, value.clone()),
            (Property::PaddingBottom, value.clone()),
            (Property::PaddingLeft, value),
        ]);
    }

    if values.len() == 2 {
        // this is margin (x, y)
        let margin_y = Value::parse(&Property::MarginRight, values[0]);
        let margin_x = Value::parse(&Property::MarginRight, values[1]);

        return Some(vec![
            (Property::PaddingTop, margin_y.clone()),
            (Property::PaddingRight, margin_x.clone()),
            (Property::PaddingBottom, margin_y),
            (Property::PaddingLeft, margin_x),
        ]);
    }

    if values.len() <= 4 {
        let margin_top = Value::parse(&Property::MarginRight, values[0]);
        let margin_right = Value::parse(&Property::MarginRight, values[1]);
        let margin_bottom = Value::parse(&Property::MarginRight, values[2]);

        if values.len() == 3 {
            return Some(vec![
                (Property::PaddingTop, margin_top),
                (Property::PaddingRight, margin_right),
                (Property::PaddingBottom, margin_bottom),
                (Property::PaddingLeft, None),
            ]);
        }

        let margin_left = Value::parse(&Property::MarginRight, values[3]);
        return Some(vec![
            (Property::PaddingTop, margin_top),
            (Property::PaddingRight, margin_right),
            (Property::PaddingBottom, margin_bottom),
            (Property::PaddingLeft, margin_left),
        ]);
    }

    None
}
