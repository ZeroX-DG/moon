use super::ExpandOutput;
use crate::property::Property;
use crate::value::Value;
use css::parser::structs::ComponentValue;

pub fn expand_border_style(values: &[&[ComponentValue]]) -> ExpandOutput {
    if values.len() == 1 {
        let value = Value::parse(&Property::BorderTopStyle, values[0]);

        if value.is_none() {
            return None;
        }

        return Some(vec![
            (Property::BorderTopStyle, value.clone()),
            (Property::BorderRightStyle, value.clone()),
            (Property::BorderBottomStyle, value.clone()),
            (Property::BorderLeftStyle, value),
        ]);
    }

    if values.len() == 2 {
        let border_y = Value::parse(&Property::BorderTopStyle, values[0]);
        let border_x = Value::parse(&Property::BorderTopStyle, values[1]);

        if border_y.is_none() || border_x.is_none() {
            return None;
        }

        return Some(vec![
            (Property::BorderTopStyle, border_y.clone()),
            (Property::BorderRightStyle, border_x.clone()),
            (Property::BorderBottomStyle, border_y),
            (Property::BorderLeftStyle, border_x),
        ]);
    }

    if values.len() <= 4 {
        let border_top = Value::parse(&Property::BorderTopStyle, values[0]);
        let border_right = Value::parse(&Property::BorderTopStyle, values[1]);
        let border_bottom = Value::parse(&Property::BorderTopStyle, values[2]);

        if border_top.is_none() || border_right.is_none() || border_bottom.is_none() {
            return None;
        }

        if values.len() == 3 {
            return Some(vec![
                (Property::BorderTopStyle, border_top),
                (Property::BorderRightStyle, border_right),
                (Property::BorderBottomStyle, border_bottom),
                (Property::BorderLeftStyle, None),
            ]);
        }

        let border_left = Value::parse(&Property::BorderTopStyle, values[3]);

        if border_left.is_none() {
            return None;
        }

        return Some(vec![
            (Property::BorderTopStyle, border_top),
            (Property::BorderRightStyle, border_right),
            (Property::BorderBottomStyle, border_bottom),
            (Property::BorderLeftStyle, border_left),
        ]);
    }

    None
}
