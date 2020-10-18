use super::selector_matching::is_match_selectors;
use css::cssom::style_rule::StyleRule;
use css::selector::structs::Specificity;
use dom::dom_ref::NodeRef;
use std::cmp::{Ord, Ordering};
use std::collections::HashMap;
use css::tokenizer::token::Token;

// values
use super::values::color::Color;
use super::values::display::Display;

type DeclaredValuesMap = HashMap<Property, Vec<PropertyDeclaration>>;

pub type Properties = HashMap<Property, Option<Value>>;

/// CSS property name
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Property {
    BackgroundColor,
    Color,
    Display,
}

/// CSS property value
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Value {
    Color(Color),
    Display(Display),
}

/// CSS property declaration for cascading
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PropertyDeclaration {
    pub value: Value,
    pub important: bool,
    pub origin: CascadeOrigin,
    pub location: CSSLocation,
    pub specificity: Specificity,
}

/// Location of the CSS applied
#[derive(Debug, Clone, Eq, PartialEq)]
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
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CascadeOrigin {
    Author,
    User,
    UserAgent,
}

/// Style rule with context for cascading
pub struct ContextualRule<'a> {
    pub inner: &'a StyleRule,
    pub origin: CascadeOrigin,
    pub location: CSSLocation,
}

macro_rules! parse_value {
    ($value:ident, $tokens:ident) => {
        if let Some(value) = $value::parse($tokens) {
            Some(Value::$value(value))
        } else {
            None
        }
    };
}

impl Value {
    pub fn parse(property: &Property, tokens: Vec<Token>) -> Option<Self> {
        match property {
            Property::BackgroundColor => parse_value!(Color, tokens),
            Property::Color => parse_value!(Color, tokens),
            Property::Display => parse_value!(Display, tokens),
        }
    }

    pub fn initial(property: &Property) -> Value {
        match property {
            Property::BackgroundColor => Value::Color(Color::transparent()),
            Property::Color => Value::Color(Color::black()),
            Property::Display => Value::Display(Display::Inline),
        }
    }
}

impl Property {
    pub fn parse(property: &str) -> Option<Self> {
        match property {
            "background-color" => Some(Property::BackgroundColor),
            "color" => Some(Property::Color),
            "display" => Some(Property::Display),
            _ => None,
        }
    }
}

/// Apply a list of style rules for a node
pub fn apply_styles(node: &NodeRef, rules: &[ContextualRule]) -> Properties {
    let mut properties = HashMap::new();

    // https://www.w3.org/TR/css3-cascade/#value-stages
    // Step 1
    let mut declared_values = collect_declared_values(&node, rules);

    // Step 2
    let cascade_values = declared_values
        .iter_mut()
        .map(|(property, values)| (property.clone(), cascade(values)))
        .collect::<Vec<(Property, Option<Value>)>>();

    for (property, value) in cascade_values {
        properties.insert(property.clone(), value.clone());
    }

    properties
}

/// Resolve specified values to computed values
pub fn compute(value: Value) -> Value {
    match value {
        _ => value
    }
}

/// Cascade sort the property declarations
/// for a property and get the wining value
fn cascade(declared_values: &mut Vec<PropertyDeclaration>) -> Option<Value> {
    declared_values.sort();

    match declared_values.first() {
        Some(declaration) => Some(declaration.value.clone()),
        _ => None,
    }
}

/// Collect declared values for each property
/// found in each style rule
fn collect_declared_values(node: &NodeRef, rules: &[ContextualRule]) -> DeclaredValuesMap {
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
                        location: rule.location.clone(),
                        specificity: rule.inner.specificity(),
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
    use crate::values::color::Color;

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
        assert_eq!(win, Some(c.value));
    }
}
