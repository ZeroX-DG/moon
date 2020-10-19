use super::render_tree::RenderNodeWeak;
use super::selector_matching::is_match_selectors;
use css::cssom::style_rule::StyleRule;
use css::parser::structs::ComponentValue;
use css::selector::structs::Specificity;
use css::tokenizer::token::Token;
use dom::dom_ref::NodeRef;
use std::cmp::{Ord, Ordering};
use std::collections::HashMap;
use strum_macros::*;

// computes
use super::computes::color::compute_color;

// values
use super::values::color::Color;
use super::values::display::Display;
use super::values::length::Length;
use super::values::percentage::Percentage;

type DeclaredValuesMap = HashMap<Property, Vec<PropertyDeclaration>>;

pub type Properties = HashMap<Property, Option<Value>>;

/// CSS property name
#[derive(Debug, Clone, Hash, Eq, PartialEq, EnumIter)]
pub enum Property {
    BackgroundColor,
    Color,
    Display,
    Width,
    Height,
    MarginTop,
    MarginRight,
    MarginBottom,
    MarginLeft,
    PaddingTop,
    PaddingRight,
    PaddingBottom,
    PaddingLeft,
}

/// CSS property value
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Value {
    Color(Color),
    Display(Display),
    Length(Length),
    Percentage(Percentage),
    Auto,
    Inherit,
    Initial,
    Unset,
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

/// Context for computing values
#[derive(Clone)]
pub struct ComputeContext<'a> {
    pub parent: &'a Option<RenderNodeWeak>,
    pub properties: HashMap<Property, Value>,
}

fn parse_keyword(tokens: &Vec<ComponentValue>, keyword: &str) -> bool {
    match tokens.iter().next() {
        Some(ComponentValue::PerservedToken(Token::Ident(word))) => {
            word.eq_ignore_ascii_case(keyword)
        }
        _ => false,
    }
}

macro_rules! parse_value {
    (Auto; $tokens:ident) => {{
        if parse_keyword($tokens, "auto") {
            Some(Value::Auto)
        } else {
            None
        }
    }};
    (Inherit; $tokens:ident) => {{
        if parse_keyword($tokens, "inherit") {
            Some(Value::Inherit)
        } else {
            None
        }
    }};
    (Initial; $tokens:ident) => {{
        if parse_keyword($tokens, "initial") {
            Some(Value::Initial)
        } else {
            None
        }
    }};
    (Unset; $tokens:ident) => {{
        if parse_keyword($tokens, "unset") {
            Some(Value::Unset)
        } else {
            None
        }
    }};
    ($value:ident; $tokens:ident) => {{
        if let Some(value) = $value::parse($tokens) {
            Some(Value::$value(value))
        } else {
            None
        }
    }};
    ($value:ident | $($remain:ident)|+; $tokens:ident) => {{
        let value = parse_value!($value; $tokens);
        if value.is_some() {
            return value;
        }
        parse_value!($($remain)|+; $tokens)
    }};
}

impl Value {
    pub fn parse(property: &Property, tokens: &Vec<ComponentValue>) -> Option<Self> {
        match property {
            Property::BackgroundColor => parse_value!(
                Color | Inherit | Initial | Unset;
                tokens
            ),
            Property::Color => parse_value!(
                Color | Inherit | Initial | Unset;
                tokens
            ),
            Property::Display => parse_value!(
                Display | Inherit | Initial | Unset;
                tokens
            ),
            Property::Width => parse_value!(
                Length | Percentage | Auto | Inherit | Initial | Unset;
                tokens
            ),
            Property::Height => parse_value!(
                Length | Percentage | Auto | Inherit | Initial | Unset;
                tokens
            ),
            Property::MarginTop => parse_value!(
                Length | Percentage | Auto | Inherit | Initial | Unset;
                tokens
            ),
            Property::MarginRight => parse_value!(
                Length | Percentage | Auto | Inherit | Initial | Unset;
                tokens
            ),
            Property::MarginBottom => parse_value!(
                Length | Percentage | Auto | Inherit | Initial | Unset;
                tokens
            ),
            Property::MarginLeft => parse_value!(
                Length | Percentage | Auto | Inherit | Initial | Unset;
                tokens
            ),
            Property::PaddingTop => parse_value!(
                Length | Percentage | Inherit | Initial | Unset;
                tokens
            ),
            Property::PaddingRight => parse_value!(
                Length | Percentage | Inherit | Initial | Unset;
                tokens
            ),
            Property::PaddingBottom => parse_value!(
                Length | Percentage | Inherit | Initial | Unset;
                tokens
            ),
            Property::PaddingLeft => parse_value!(
                Length | Percentage | Inherit | Initial | Unset;
                tokens
            ),
        }
    }

    pub fn initial(property: &Property) -> Value {
        match property {
            Property::BackgroundColor => Value::Color(Color::transparent()),
            Property::Color => Value::Color(Color::black()),
            Property::Display => Value::Display(Display::Inline),
            Property::Width => Value::Auto,
            Property::Height => Value::Auto,
            Property::MarginTop => Value::Length(Length::zero()),
            Property::MarginRight => Value::Length(Length::zero()),
            Property::MarginBottom => Value::Length(Length::zero()),
            Property::MarginLeft => Value::Length(Length::zero()),
            Property::PaddingTop => Value::Length(Length::zero()),
            Property::PaddingRight => Value::Length(Length::zero()),
            Property::PaddingBottom => Value::Length(Length::zero()),
            Property::PaddingLeft => Value::Length(Length::zero()),
        }
    }
}

impl Property {
    pub fn parse(property: &str) -> Option<Self> {
        match property {
            "background-color" => Some(Property::BackgroundColor),
            "color" => Some(Property::Color),
            "display" => Some(Property::Display),
            "width" => Some(Property::Width),
            "height" => Some(Property::Height),
            "margin-top" => Some(Property::MarginTop),
            "margin-right" => Some(Property::MarginRight),
            "margin-bottom" => Some(Property::MarginBottom),
            "margin-left" => Some(Property::MarginLeft),
            "padding-top" => Some(Property::PaddingTop),
            "padding-right" => Some(Property::PaddingRight),
            "padding-bottom" => Some(Property::PaddingBottom),
            "padding-left" => Some(Property::PaddingLeft),
            _ => None,
        }
    }
}

/// Apply a list of style rules for a node
pub fn apply_styles(node: &NodeRef, rules: &[ContextualRule]) -> Properties {
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

/// Resolve specified values to computed values
pub fn compute(property: &Property, value: &Value, context: &ComputeContext) -> Value {
    match value {
        Value::Color(_) => compute_color(value, property, context),
        _ => value.clone(),
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
                let values = &declaration.value;
                let value = Value::parse(&property, values);

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
    use css::parser::structs::ComponentValue;
    use css::tokenizer::token::Token;

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
        assert_eq!(value_percentage, Some(Value::Percentage(Percentage(20.5))));
    }
}
