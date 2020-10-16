use super::selector_matching::is_match_selectors;
use super::style_tree::{Properties, Property, Value};
use css::cssom::style_rule::StyleRule;
use dom::dom_ref::NodeRef;
use std::collections::HashMap;

type DeclaredValuesMap = HashMap<Property, Vec<PropertyDeclaration>>;

/// CSS property declaration for cascading
#[derive(Debug)]
pub struct PropertyDeclaration {
    pub value: Value,
    pub important: bool,
    pub origin: CascadeOrigin,
    pub location: CSSLocation
}

/// Location of the CSS applied
#[derive(Debug, Clone)]
pub enum CSSLocation {
    /// Inline CSS (in HTML tags)
    Inline,
    /// Embedded CSS (in HTML style tag)
    Embedded,
    /// External CSS (in external css file)
    External,
}

/// Cascade origin
/// https://www.w3.org/TR/css-cascade-4/#origin
#[derive(Debug, Clone)]
pub enum CascadeOrigin {
    Author,
    User,
    UserAgent,
    Animation,
    Transition,
}

/// Style rule with context for cascading
pub struct ContextualRule<'a> {
    pub inner: &'a StyleRule,
    pub origin: CascadeOrigin,
    pub location: CSSLocation
}

pub fn apply_styles(node: &NodeRef, rules: &[ContextualRule]) -> Properties {
    let mut properties = HashMap::new();

    // https://www.w3.org/TR/css3-cascade/#value-stages
    // Step 1
    let declared_values = collect_declared_values(&node, rules);

    // Step 2
    let cascade_values = declared_values
        .keys()
        .map(|property| {
            (
                property.clone(),
                cascade(property, declared_values.get(property)),
            )
        })
        .collect::<Vec<(Property, Option<Value>)>>();

    for (property, value) in cascade_values {
        properties.insert(property, value);
    }

    properties
}

fn cascade(
    property: &Property,
    declared_values: Option<&Vec<PropertyDeclaration>>,
) -> Option<Value> {
    if let Some(declared_values) = declared_values {
        // TODO: perform cascade
    }
    None
}

fn collect_declared_values(
    node: &NodeRef,
    rules: &[ContextualRule]
) -> DeclaredValuesMap {
    let mut result: DeclaredValuesMap = HashMap::new();

    let matched_rules = rules
        .iter()
        .filter(|rule| is_match_selectors(node, &rule.inner.selectors))
        .collect::<Vec<&ContextualRule>>();

    for rule in matched_rules {
        for declaration in &rule.inner.declarations {
            let property = Property::parse(&declaration.name);

            if let Some(property) = property {
                let tokens = declaration.tokens();
                let value = Value::parse(&property, tokens);

                if let Some(value) = value {
                    let declaration = PropertyDeclaration {
                        value,
                        important: declaration.important,
                        origin: rule.origin.clone(),
                        location: rule.location.clone()
                    };
                    if result.contains_key(&property) {
                        result.get_mut(&property).unwrap().push(declaration);
                    } else {
                        result.insert(property, vec![declaration]);
                    }
                }
            }
        }
    }

    result
}
