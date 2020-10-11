use std::collections::HashMap;
use dom::dom_ref::NodeRef;
use css::cssom::css_rule::CSSRule;
use css::cssom::style_rule::StyleRule;
use css::cssom::stylesheet::StyleSheet;
use css::tokenizer::token::Token;
use css::parser::structs::ComponentValue;
use super::selector_matching::is_match_selectors;

// values
use super::values::color::Color;
use super::values::display::Display;

pub type Properties = HashMap<Property, Value>;

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum Property {
    BackgroundColor,
    Color,
    Display
}

#[derive(Debug, Clone)]
pub enum Value {
    Color(Color),
    Display(Display)
}

#[derive(Debug)]
pub struct StyleNode {
    pub node: NodeRef,
    pub properties: Properties,
    pub children: Vec<StyleNode>
}

impl StyleNode {
    pub fn get_value(&self, prop: Property) -> Value {
        self.properties.get(&prop).cloned().unwrap_or(Value::default(&prop))
    }
}

impl Value {
    pub fn parse(property: &Property, tokens: Vec<Token>) -> Option<Self> {
        match property {
            Property::BackgroundColor => Color::parse(tokens),
            Property::Color => Color::parse(tokens),
            Property::Display => Display::parse(tokens),
        }
    }

    pub fn default(property: &Property) -> Value {
        match property {
            Property::BackgroundColor => Color::default(),
            Property::Color => Color::default(),
            Property::Display => Display::default()
        }
    }
}

impl Property {
    pub fn parse(property: &str) -> Option<Self> {
        match property {
            "background-color" => Some(Property::BackgroundColor),
            "color" => Some(Property::Color),
            "display" => Some(Property::Display),
            _ => None
        }
    }
}

fn apply_stylesheets(node: NodeRef, stylesheets: &Vec<StyleSheet>) -> Properties {
    let mut properties = HashMap::new();

    for stylesheet in stylesheets {
        for rule in stylesheet.iter() {
            match rule {
                CSSRule::Style(style_rule) => apply_style_rule(&node, style_rule, &mut properties)
            }
        }
    }

    properties
}

fn apply_style_rule(node: &NodeRef, rule: &StyleRule, properties: &mut Properties) {
    if is_match_selectors(&node, &rule.selectors) {
        for declaration in &rule.declarations {
            let property = Property::parse(&declaration.name);

            if let Some(property) = property {
                let tokens = declaration.value
                    .clone()
                    .into_iter()
                    .filter_map(|com| {
                        match com {
                            ComponentValue::PerservedToken(t) => Some(t),
                            _ => None
                        }
                    })
                    .collect();
                if let Some(value) = Value::parse(&property, tokens) {
                    properties.insert(property, value);
                }
            }
        }
    }
}

pub fn build_style_tree(node: NodeRef, stylesheets: &Vec<StyleSheet>) -> StyleNode {
    let properties = if node.is::<dom::text::Text>() {
        HashMap::new()
    } else {
        apply_stylesheets(node.clone(), stylesheets)
    };
    StyleNode {
        node: node.clone(),
        properties,
        children: node
            .borrow()
            .as_node()
            .child_nodes()
            .into_iter() // this is fine because we clone the node when iterate
            .map(|child| build_style_tree(child, stylesheets))
            .collect()
    }
}
