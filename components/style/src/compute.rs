use std::collections::HashMap;

use dom::node::NodePtr;
use style_types::{ContextualRule, Property, Value};

use crate::cascade::collect_cascaded_values;

pub fn compute_styles(node: NodePtr, rules: &[ContextualRule]) -> HashMap<Property, Value> {
    let mut styles = collect_cascaded_values(&node, rules);

    compute_default_values(&node, &mut styles);
    styles
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
