use super::selector_matching::is_match_selectors;
use super::style_tree::{Properties, Property, Value};
use css::cssom::style_rule::StyleRule;
use dom::dom_ref::NodeRef;
use std::cmp::{Ord, Ordering};
use std::collections::HashMap;

type DeclaredValuesMap = HashMap<Property, Vec<PropertyDeclaration>>;

/// CSS property declaration for cascading
#[derive(Debug, Eq, PartialEq)]
pub struct PropertyDeclaration {
    pub value: Value,
    pub important: bool,
    pub origin: CascadeOrigin,
    pub location: CSSLocation,
    pub specificity: usize,
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

fn cascade(declared_values: &mut Vec<PropertyDeclaration>) -> Option<Value> {
    declared_values.sort();

    match declared_values.first() {
        Some(declaration) => Some(declaration.value.clone()),
        _ => None,
    }
}

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
                        specificity: 0,
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
