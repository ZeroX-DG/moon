use std::collections::HashMap;

use super::border_color::expand_border_color;
use super::border_style::expand_border_style;
use super::border_width::expand_border_width;
use super::ExpandOutput;
use css::parser::structs::ComponentValue;
use style_types::{Property, Value};

pub fn expand_border(values: &[&[ComponentValue]]) -> ExpandOutput {
    let mut expanded_styles = None;
    let mut expanded_widths = None;
    let mut expanded_colors = None;

    for tokens in values {
        if let Some(style) = expand_border_style(&[tokens]) {
            if expanded_styles.is_none() {
                expanded_styles = Some(style);
                continue;
            } else {
                return None;
            }
        }
        if let Some(width) = expand_border_width(&[tokens]) {
            if expanded_widths.is_none() {
                expanded_widths = Some(width);
                continue;
            } else {
                return None;
            }
        }
        if let Some(color) = expand_border_color(&[tokens]) {
            if expanded_colors.is_none() {
                expanded_colors = Some(color);
                continue;
            } else {
                return None;
            }
        }
    }

    let mut result = vec![];

    if let Some(style) = expanded_styles {
        result.extend(style);
    }

    if let Some(width) = expanded_widths {
        result.extend(width);
    }

    if let Some(color) = expanded_colors {
        result.extend(color);
    }

    if result.len() == 1 {
        return None;
    }

    Some(result)
}

pub fn expand_border_top(values: &[&[ComponentValue]]) -> ExpandOutput {
    expand_single_border(values, "top")
}

pub fn expand_border_right(values: &[&[ComponentValue]]) -> ExpandOutput {
    expand_single_border(values, "right")
}

pub fn expand_border_bottom(values: &[&[ComponentValue]]) -> ExpandOutput {
    expand_single_border(values, "bottom")
}

pub fn expand_border_left(values: &[&[ComponentValue]]) -> ExpandOutput {
    expand_single_border(values, "left")
}

fn expand_single_border(values: &[&[ComponentValue]], property: &str) -> ExpandOutput {
    let mut expanded_style = None;
    let mut expanded_width = None;
    let mut expanded_color = None;

    let map = HashMap::from([
        (
            "top",
            [
                Property::BorderTopStyle,
                Property::BorderTopWidth,
                Property::BorderTopColor,
            ],
        ),
        (
            "right",
            [
                Property::BorderRightStyle,
                Property::BorderRightWidth,
                Property::BorderRightColor,
            ],
        ),
        (
            "bottom",
            [
                Property::BorderBottomStyle,
                Property::BorderBottomWidth,
                Property::BorderBottomColor,
            ],
        ),
        (
            "left",
            [
                Property::BorderLeftStyle,
                Property::BorderLeftWidth,
                Property::BorderLeftColor,
            ],
        ),
    ]);

    for tokens in values {
        if let Some(style) = Value::parse(&Property::BorderTopStyle, tokens) {
            if expanded_style.is_none() {
                expanded_style = Some(style);
                continue;
            } else {
                return None;
            }
        }
        if let Some(width) = Value::parse(&Property::BorderTopWidth, tokens) {
            if expanded_width.is_none() {
                expanded_width = Some(width);
                continue;
            } else {
                return None;
            }
        }
        if let Some(color) = Value::parse(&Property::BorderTopColor, tokens) {
            if expanded_color.is_none() {
                expanded_color = Some(color);
                continue;
            } else {
                return None;
            }
        }
    }

    let mut result = vec![];

    result.extend([(map[property][0].clone(), expanded_style)]);
    result.extend([(map[property][1].clone(), expanded_width)]);
    result.extend([(map[property][2].clone(), expanded_color)]);

    if result.len() == 1 {
        return None;
    }

    Some(result)
}
