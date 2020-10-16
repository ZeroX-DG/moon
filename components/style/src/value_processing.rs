use super::selector_matching::is_match_selectors;
use super::style_tree::{Properties, Property, Value};
use css::cssom::css_rule::CSSRule;
use css::cssom::stylesheet::StyleSheet;
use css::parser::structs::ComponentValue;
use dom::dom_ref::NodeRef;
use std::collections::HashMap;

type DeclaredValuesMap = HashMap<Property, Vec<PropertyDeclaration>>;

/// CSS property declaration for cascading
#[derive(Debug)]
pub struct PropertyDeclaration {
    pub value: Value,
    pub important: bool,
    pub origin: CascadeOrigin,
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

pub fn apply_styles(node: &NodeRef, stylesheets: &[(StyleSheet, CascadeOrigin)]) -> Properties {
    let mut properties = HashMap::new();

    // https://www.w3.org/TR/css3-cascade/#value-stages
    // Step 1
    let declared_values = collect_declared_values(&node, stylesheets);

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
    stylesheets: &[(StyleSheet, CascadeOrigin)],
) -> DeclaredValuesMap {
    let mut result: DeclaredValuesMap = HashMap::new();

    stylesheets.iter().for_each(|(stylesheet, origin)| {
        let matched_rules = stylesheet
            .iter()
            .filter_map(|rule| match rule {
                CSSRule::Style(style_rule) => {
                    if is_match_selectors(node, &style_rule.selectors) {
                        return Some((rule, origin));
                    }
                    None
                }
            })
            .collect::<Vec<(&CSSRule, &CascadeOrigin)>>();

        matched_rules.iter().for_each(|(rule, origin)| match rule {
            CSSRule::Style(style_rule) => {
                style_rule.declarations.iter().for_each(|declaration| {
                    let property = Property::parse(&declaration.name);

                    if let Some(property) = property {
                        let tokens = declaration
                            .value
                            .clone()
                            .into_iter()
                            .filter_map(|com| match com {
                                ComponentValue::PerservedToken(t) => Some(t),
                                _ => None,
                            })
                            .collect();

                        let value = Value::parse(&property, tokens);
                        if let Some(value) = value {
                            let declaration = PropertyDeclaration {
                                value,
                                important: declaration.important,
                                origin: (*origin).clone(),
                            };
                            if result.contains_key(&property) {
                                result.get_mut(&property).unwrap().push(declaration);
                            } else {
                                result.insert(property, vec![declaration]);
                            }
                        }
                    }
                });
            }
        });
    });

    result
}
