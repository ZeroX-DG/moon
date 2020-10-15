use super::selector_matching::is_match_selectors;
use super::style_tree::{Properties, Property, Value};
use css::cssom::css_rule::CSSRule;
use css::cssom::stylesheet::StyleSheet;
use css::parser::structs::ComponentValue;
use dom::dom_ref::NodeRef;
use std::collections::HashMap;
use strum::IntoEnumIterator;

type DeclaredValuesMap = HashMap<Property, Vec<PropertyDeclaration>>;

/// CSS property declaration for cascading
pub struct PropertyDeclaration {
    pub value: Value,
    pub important: bool,
}

pub fn apply_styles(node: &NodeRef, stylesheets: &[StyleSheet]) -> Properties {
    let mut properties = HashMap::new();

    // https://www.w3.org/TR/css3-cascade/#value-stages
    // Step 1
    let declared_values = collect_declared_values(&node, stylesheets);

    // Step 2
    let cascade_values = Property::iter()
        .map(|property| {
            (
                property.clone(),
                cascade(&property, declared_values.get(&property)),
            )
        })
        .collect::<Vec<(Property, Option<Value>)>>();

    properties
}

fn cascade(property: &Property, declared_values: Option<&Vec<PropertyDeclaration>>) -> Option<Value> {
    if let Some(declared_values) = declared_values {
        // TODO: perform cascade
    }
    None
}

fn collect_declared_values(node: &NodeRef, stylesheets: &[StyleSheet]) -> DeclaredValuesMap {
    let mut result: DeclaredValuesMap = HashMap::new();

    stylesheets.iter().for_each(|stylesheet| {
        let matched_rules = stylesheet
            .iter()
            .filter(|rule| match rule {
                CSSRule::Style(style_rule) => {
                    return is_match_selectors(node, &style_rule.selectors)
                }
            })
            .collect::<Vec<&CSSRule>>();

        matched_rules.iter().for_each(|rule| match rule {
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
