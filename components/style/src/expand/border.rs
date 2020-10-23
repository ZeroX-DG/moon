use super::border_color::expand_border_color;
use super::border_style::expand_border_style;
use super::border_width::expand_border_width;
use super::ExpandOutput;
use css::parser::structs::ComponentValue;

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
