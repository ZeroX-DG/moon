use std::collections::HashMap;
use dom::dom_ref::NodeRef;
use css::cssom::style_rule::StyleRule;
use css::tokenizer::token::Token;
use css::parser::structs::ComponentValue;

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
    node: NodeRef,
    properties: Properties,
    children: Vec<StyleNode>
}

impl StyleNode {
    pub fn use_style_rule(&mut self, rule: StyleRule) {
        for declaration in rule.declarations {
            let property = match declaration.name.as_ref() {
                "background-color" => Some(Property::BackgroundColor),
                "color" => Some(Property::Color),
                "display" => Some(Property::Display),
                _ => None
            };

            if let Some(property) = property {
                let tokens = declaration.value
                    .into_iter()
                    .filter_map(|com| {
                        match com {
                            ComponentValue::PerservedToken(t) => Some(t),
                            _ => None
                        }
                    })
                    .collect();
                if let Some(value) = Value::parse(&property, tokens) {
                    self.properties.insert(property, value);
                }
            }
        }
    }

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
