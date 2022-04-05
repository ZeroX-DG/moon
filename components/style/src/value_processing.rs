use crate::computes::border_radius::compute_border_radius;
use crate::computes::border_width::compute_border_width;
use crate::computes::margin::compute_margin;
use crate::computes::padding::compute_padding;
use crate::property::Property;
use crate::render_tree::RenderNode;
use crate::value::Value;
use crate::values::length::Length;
use crate::values::length::LengthUnit;

use super::selector_matching::is_match_selectors;
use css::parser::structs::ComponentValue;
use css::parser::structs::Declaration;
use css::selector::structs::Specificity;
use css::tokenizer::token::Token;
use dom::node::NodePtr;
use shared::tree_node::WeakTreeNode;
use std::borrow::Borrow;
use std::cmp::{Ord, Ordering};
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::rc::Rc;
use style_types::CSSLocation;
use style_types::CascadeOrigin;
use style_types::ContextualRule;

use super::expand::prelude::*;

// computes
use super::computes::color::compute_color;
use super::computes::font_size::compute_font_size;

type DeclaredValuesMap = HashMap<Property, Vec<PropertyDeclaration>>;

pub type Properties = HashMap<Property, Option<Value>>;

/// CSS property declaration for cascading
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PropertyDeclaration {
    pub value: Value,
    pub important: bool,
    pub origin: CascadeOrigin,
    pub location: CSSLocation,
    pub specificity: Specificity,
}

/// Context for computing values
pub struct ComputeContext<'a> {
    pub parent: Option<WeakTreeNode<RenderNode>>,
    pub properties: HashMap<Property, Value>,
    pub style_cache: &'a mut StyleCache,
}

#[derive(Debug)]
pub struct StyleCache(HashSet<ValueRef>);

impl StyleCache {
    pub fn new() -> Self {
        Self(HashSet::new())
    }

    pub fn get(&mut self, value: &Value) -> ValueRef {
        if !self.0.contains(value) {
            self.0.insert(ValueRef::new(value.clone()));
        }
        self.0.get(value).unwrap().clone()
    }
}

// TODO: drop the value from cache when rc is dropped to 1
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ValueRef(pub Rc<Value>);

impl Borrow<Value> for ValueRef {
    fn borrow(&self) -> &Value {
        self.0.borrow()
    }
}

impl ValueRef {
    pub fn new(value: Value) -> Self {
        Self(Rc::new(value))
    }

    pub fn inner(&self) -> &Value {
        self.borrow()
    }

    pub fn is_auto(&self) -> bool {
        match self.borrow() {
            Value::Auto => true,
            _ => false,
        }
    }

    pub fn to_px(&self, relative_to: f32) -> f32 {
        match self.borrow() {
            Value::Length(l) => l.to_px(relative_to),
            Value::Percentage(p) => p.to_px(relative_to),
            Value::BorderWidth(w) => w.to_px(),
            Value::Auto => 0.,
            _ => unreachable!("Invalid call to_px on invalid value: {:?}", self),
        }
    }

    pub fn to_absolute_px(&self) -> f32 {
        match self.borrow() {
            Value::Length(Length {
                value,
                unit: LengthUnit::Px,
            }) => **value,
            _ => unimplemented!("Calling to_absolute_px for unsupported value"),
        }
    }

    pub fn map<T, F>(&self, map_fn: F) -> T
    where
        F: FnOnce(&Value) -> T,
    {
        map_fn(self.inner())
    }
}

impl Clone for ValueRef {
    fn clone(&self) -> Self {
        ValueRef(self.0.clone())
    }
}

impl Deref for ValueRef {
    type Target = Rc<Value>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Apply a list of style rules for a node
pub fn apply_styles(node: &NodePtr, rules: &[ContextualRule]) -> Properties {
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
pub fn compute(property: &Property, value: &Value, context: &mut ComputeContext) -> ValueRef {
    // TODO: Some of these compute functions is quite similar. For example: compute_margin & compute_padding.
    // We should optimize this by computing base on value instead of property.
    match property {
        Property::Color => compute_color(value, context),
        Property::FontSize => compute_font_size(value, context),
        Property::MarginTop
        | Property::MarginLeft
        | Property::MarginRight
        | Property::MarginBottom => compute_margin(value, context),
        Property::BorderTopWidth
        | Property::BorderLeftWidth
        | Property::BorderBottomWidth
        | Property::BorderRightWidth => compute_border_width(property, value, context),
        Property::BorderTopLeftRadius
        | Property::BorderTopRightRadius
        | Property::BorderBottomLeftRadius
        | Property::BorderBottomRightRadius => compute_border_radius(value, context),
        Property::PaddingTop
        | Property::PaddingLeft
        | Property::PaddingRight
        | Property::PaddingBottom => compute_padding(value, context),
        _ => context.style_cache.get(value),
    }
}

/// Cascade sort the property declarations
/// for a property and get the wining value
fn cascade(declared_values: &mut Vec<PropertyDeclaration>) -> Option<Value> {
    declared_values.sort();

    match declared_values.last() {
        Some(declaration) => Some(declaration.value.clone()),
        _ => None,
    }
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
    use crate::values::color::Color;
    use crate::values::prelude::Percentage;
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
        assert_eq!(win, Some(b.value));
    }
}
