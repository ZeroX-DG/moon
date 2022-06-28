use super::selector_matching::is_match_selectors;
use css::parser::structs::ComponentValue;
use css::parser::structs::Declaration;
use css::selector::structs::Specificity;
use css::tokenizer::token::Token;
use dom::node::NodePtr;
use std::cmp::{Ord, Ordering};
use std::collections::HashMap;
use style_types::CSSLocation;
use style_types::CascadeOrigin;
use style_types::ContextualRule;
use style_types::Property;
use style_types::Value;

use super::expand::prelude::*;

type DeclaredValuesMap = HashMap<Property, Vec<PropertyDeclaration>>;

pub type Properties = HashMap<Property, Value>;

/// CSS property declaration for cascading
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PropertyDeclaration {
    pub value: Value,
    pub important: bool,
    pub origin: CascadeOrigin,
    pub location: CSSLocation,
    pub specificity: Specificity,
}

pub fn collect_cascaded_values(node: &NodePtr, rules: &[ContextualRule]) -> Properties {
    // https://www.w3.org/TR/css3-cascade/#value-stages
    // Step 1
    let mut declared_values = collect_declared_values(&node, rules);

    // Step 2
    let cascade_values = declared_values
        .iter_mut()
        .map(|(property, values)| (property.clone(), cascade(values)))
        .collect::<Properties>();

    cascade_values
}

/// Cascade sort the property declarations
/// for a property and get the wining value
fn cascade(declared_values: &mut Vec<PropertyDeclaration>) -> Value {
    declared_values.sort();
    declared_values.last().unwrap().value.clone()
}

/// Get a short-hand property expander
fn get_expander_shorthand_property(
    property: &str,
) -> Option<&dyn Fn(&[&[ComponentValue]]) -> ExpandOutput> {
    match property {
        "margin" => Some(&expand_margin),
        "padding" => Some(&expand_padding),
        "border" => Some(&expand_border),
        "border-style" => Some(&expand_border_style),
        "border-width" => Some(&expand_border_width),
        "border-color" => Some(&expand_border_color),
        "border-radius" => Some(&expand_border_radius),
        "border-top" => Some(&expand_border_top),
        "border-right" => Some(&expand_border_right),
        "border-bottom" => Some(&expand_border_bottom),
        "border-left" => Some(&expand_border_left),
        _ => None,
    }
}

/// Collect declared values for each property
/// found in each style rule
fn collect_declared_values(node: &NodePtr, rules: &[ContextualRule]) -> DeclaredValuesMap {
    let mut result: DeclaredValuesMap = HashMap::new();

    if !node.is_element() {
        return result;
    }

    let matched_rules = rules
        .iter()
        .filter(|rule| is_match_selectors(node, &rule.inner.selectors))
        .collect::<Vec<&ContextualRule>>();

    let mut insert_declaration =
        |value: Value, property: Property, rule: &ContextualRule, declaration: &Declaration| {
            let declaration = PropertyDeclaration {
                value,
                important: declaration.important,
                origin: rule.origin.clone(),
                location: rule.location.clone(),
                specificity: rule.inner.specificity(),
            };
            if result.contains_key(&property) {
                result.get_mut(&property).unwrap().push(declaration);
            } else {
                result.insert(property, vec![declaration]);
            }
        };

    for rule in matched_rules {
        for declaration in &rule.inner.declarations {
            if let Some(expand) = get_expander_shorthand_property(&declaration.name) {
                // process short hand property
                let tokens = declaration
                    .value
                    .split(|val| match val {
                        ComponentValue::PerservedToken(Token::Whitespace) => true,
                        _ => false,
                    })
                    .collect::<Vec<&[ComponentValue]>>();

                if let Some(values) = expand(&tokens) {
                    for (property, value) in values {
                        if let Some(v) = value {
                            insert_declaration(v, property, rule, declaration);
                        }
                    }
                }
            } else {
                // process long hand css property
                let property = Property::parse(&declaration.name);
                if let Some(property) = property {
                    let values = &declaration.value;
                    let value = Value::parse(&property, values);

                    if let Some(value) = value {
                        insert_declaration(value, property, rule, declaration);
                    }
                }
            }
        }
    }

    result
}

/// The implementation for ordering for cascade sort
///
/// These are the steps to compare the order:
/// 1. Comparing the location of the property declaration (Inline, Embedded, etc.)
/// 2. If step 1 result in equal ordering compare the cascade origin
/// 3. If step 2 result in equal ordering compare the specificity
impl Ord for PropertyDeclaration {
    fn cmp(&self, other: &Self) -> Ordering {
        match cmp_location(self, other) {
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
            Ordering::Equal => match cmp_cascade_origin(self, other) {
                Ordering::Greater => Ordering::Greater,
                Ordering::Less => Ordering::Less,
                Ordering::Equal => self.specificity.cmp(&other.specificity),
            },
        }
    }
}

impl PartialOrd for PropertyDeclaration {
    fn partial_cmp(&self, other: &PropertyDeclaration) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn cmp_location(a: &PropertyDeclaration, b: &PropertyDeclaration) -> Ordering {
    match (&a.location, &b.location) {
        (CSSLocation::Inline, CSSLocation::Embedded)
        | (CSSLocation::Inline, CSSLocation::External)
        | (CSSLocation::Embedded, CSSLocation::External) => Ordering::Greater,
        (CSSLocation::Inline, CSSLocation::Inline)
        | (CSSLocation::Embedded, CSSLocation::Embedded)
        | (CSSLocation::External, CSSLocation::External) => Ordering::Equal,
        _ => Ordering::Less,
    }
}

/// Comparing cascade origin and importance
///
/// 1. Transition declarations [css-transitions-1]
/// 2. Important user agent declarations
/// 3. Important user declarations
/// 4. Important author declarations
/// 5. Animation declarations [css-animations-1]
/// 6. Normal author declarations
/// 7. Normal user declarations
/// 8. Normal user agent declarations
fn cmp_cascade_origin(a: &PropertyDeclaration, b: &PropertyDeclaration) -> Ordering {
    // -----------------
    // Rule #2 #3 #4 #5 #6 #7 #8
    match (a.important, b.important) {
        (true, false) => return Ordering::Greater,
        (false, true) => return Ordering::Less,
        // #2 #3 #4
        (true, true) => {
            return match (&a.origin, &b.origin) {
                (CascadeOrigin::UserAgent, CascadeOrigin::User)
                | (CascadeOrigin::User, CascadeOrigin::Author)
                | (CascadeOrigin::UserAgent, CascadeOrigin::Author) => Ordering::Greater,
                (CascadeOrigin::Author, CascadeOrigin::Author)
                | (CascadeOrigin::User, CascadeOrigin::User)
                | (CascadeOrigin::UserAgent, CascadeOrigin::UserAgent) => Ordering::Equal,
                _ => Ordering::Less,
            }
        }
        // #5 #6 #7 #8
        (false, false) => {
            // #6 #7 #8
            return match (&a.origin, &b.origin) {
                (CascadeOrigin::Author, CascadeOrigin::User)
                | (CascadeOrigin::User, CascadeOrigin::UserAgent)
                | (CascadeOrigin::Author, CascadeOrigin::UserAgent) => Ordering::Greater,
                (CascadeOrigin::Author, CascadeOrigin::Author)
                | (CascadeOrigin::User, CascadeOrigin::User)
                | (CascadeOrigin::UserAgent, CascadeOrigin::UserAgent) => Ordering::Equal,
                _ => Ordering::Less,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use css::parser::structs::ComponentValue;
    use css::tokenizer::token::Token;
    use style_types::values::prelude::{Color, Percentage};

    #[test]
    fn cascade_simple() {
        let a = PropertyDeclaration {
            location: CSSLocation::External,
            origin: CascadeOrigin::User,
            important: false,
            value: Value::Color(Color::black()),
            specificity: Specificity::new(1, 0, 1),
        };

        let b = PropertyDeclaration {
            location: CSSLocation::Inline,
            origin: CascadeOrigin::User,
            important: false,
            value: Value::Color(Color::black()),
            specificity: Specificity::new(1, 0, 1),
        };

        let c = PropertyDeclaration {
            location: CSSLocation::Embedded,
            origin: CascadeOrigin::User,
            important: true,
            value: Value::Color(Color::black()),
            specificity: Specificity::new(1, 0, 1),
        };

        let mut declared = vec![a.clone(), b.clone(), c.clone()];

        let win = cascade(&mut declared);
        assert_eq!(win, c.value);
    }

    #[test]
    fn parse_multiple_value_types() {
        let tokens_auto = vec![ComponentValue::PerservedToken(Token::Ident(
            "auto".to_string(),
        ))];
        let value_auto = Value::parse(&Property::Width, &tokens_auto);

        let tokens_percentage = vec![ComponentValue::PerservedToken(Token::Percentage(20.5))];
        let value_percentage = Value::parse(&Property::Width, &tokens_percentage);

        let tokens_inherit = vec![ComponentValue::PerservedToken(Token::Ident(
            "inherit".to_string(),
        ))];
        let value_inherit = Value::parse(&Property::Width, &tokens_inherit);

        assert_eq!(value_auto, Some(Value::Auto));
        assert_eq!(value_inherit, Some(Value::Inherit));
        assert_eq!(
            value_percentage,
            Some(Value::Percentage(Percentage(20.5.into())))
        );
    }

    #[test]
    fn parse_multiple_value_override() {
        let a = PropertyDeclaration {
            location: CSSLocation::External,
            origin: CascadeOrigin::User,
            important: false,
            value: Value::Color(Color::black()),
            specificity: Specificity::new(0, 0, 0),
        };

        let b = PropertyDeclaration {
            location: CSSLocation::External,
            origin: CascadeOrigin::User,
            important: false,
            value: Value::Color(Color::transparent()),
            specificity: Specificity::new(0, 0, 1),
        };

        let mut declared = vec![b.clone(), a.clone()];

        let win = cascade(&mut declared);
        assert_eq!(win, b.value);
    }
}
