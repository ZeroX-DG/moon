use std::collections::HashMap;

use dom::node::NodePtr;
use style_types::{ContextualRule, Property, Value, values::{length::LengthUnit, prelude::{Color, Length, Percentage, BorderStyle}}};

use crate::cascade::collect_cascaded_values;

pub fn compute_styles(node: NodePtr, rules: &[ContextualRule]) -> HashMap<Property, Value> {
    let mut styles = collect_cascaded_values(&node, rules);

    compute_default_values(&node, &mut styles);
    compute_absolute_values(&node, &mut styles);
    styles
}

fn compute_absolute_values(node: &NodePtr, styles: &mut HashMap<Property, Value>) {
    let base_font_size = 16.;
    let parent_font_size = node
        .parent()
        .map(|parent| NodePtr(parent).get_style(&Property::FontSize).to_absolute_px())
        .unwrap_or(base_font_size); 

    let root_font_size = node
        .owner_document()
        .map(|root| NodePtr(root).get_style(&Property::FontSize).to_absolute_px())
        .unwrap_or(base_font_size);

    let mut updates = Vec::new();
    for (property, value) in styles.iter() {
        match value {
            Value::Length(length) => match length {
                Length { value, unit: LengthUnit::Em } => {
                    let abs_length = Length::new_px(value.0 * parent_font_size);
                    updates.push((property.clone(), Value::Length(abs_length)));
                },
                Length { value, unit: LengthUnit::Rem } => {
                    let abs_length = Length::new_px(value.0 * root_font_size);
                    updates.push((property.clone(), Value::Length(abs_length)));
                },
                _ => {}
            }
            Value::Percentage(percentage) => match percentage {
                Percentage(value) if matches!(property, Property::FontSize) => {
                    let abs_length = Length::new_px(value.0 * parent_font_size / 100.);
                    updates.push((property.clone(), Value::Length(abs_length)));
                }
                _ => {}
            }
            Value::Color(color) => match color {
                Color::CurrentColor => {
                    let color = node.parent()
                        .map(|p| p.get_style(&Property::Color))
                        .unwrap_or(Value::initial(&Property::Color));
                    updates.push((property.clone(), color));
                }
                _ => {}
            }
            Value::BorderWidth(_) => {
                let border_style = match &property {
                    Property::BorderTopWidth => Property::BorderTopStyle,
                    Property::BorderLeftWidth => Property::BorderLeftStyle,
                    Property::BorderBottomWidth => Property::BorderBottomStyle,
                    Property::BorderRightWidth => Property::BorderRightStyle,
                    _ => unreachable!(),
                };

                let border_style = styles.get(&border_style).unwrap();
                match border_style {
                    Value::BorderStyle(BorderStyle::None) | Value::BorderStyle(BorderStyle::Hidden) => {
                        let value = Value::Length(Length::zero());
                        updates.push((property.clone(), value));
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    for (property, value) in updates {
        styles.insert(property, value);
    }
}

fn compute_default_values(node: &NodePtr, styles: &mut HashMap<Property, Value>) {
    // get inherit value for a property
    let inherit = |property: Property| {
        if let Some(parent) = &node.parent() {
            return (
                property.clone(),
                parent.get_style(&property),
            )
        }
        // if there's no parent
        // we will use the initial value for that property
        return (property.clone(), Value::initial(&property));
    };

    // Step 3
    let specified_values = Property::all()
        .map(|property| {
            if let Some(value) = styles.get(&property) {
                // Explicit defaulting
                // https://www.w3.org/TR/css3-cascade/#defaulting-keywords
                if let Value::Initial = value {
                    return (property.clone(), Value::initial(&property));
                }
                if let Value::Inherit = value {
                    return inherit(property);
                }
                if let Value::Unset = value {
                    if property.inheritable() {
                        return inherit(property);
                    }
                    return (property.clone(), Value::initial(&property));
                }
                return (property, value.clone());
            }
            // if there's no specified value in properties
            // we will try to inherit it
            if property.inheritable() {
                return inherit(property);
            }
            // if the property is not inheritable
            // we will use the initial value for that property
            return (property.clone(), Value::initial(&property));
        })
        .collect::<HashMap<Property, Value>>();

    for (property, value) in specified_values {
        styles.insert(property, value);
    }
}
