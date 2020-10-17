use super::value_processing::{apply_styles, ContextualRule};
use css::tokenizer::token::Token;
use dom::dom_ref::NodeRef;
use std::collections::HashMap;

// values
use super::values::color::Color;
use super::values::display::Display;

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

/// A style node in the style tree
#[derive(Debug)]
pub struct RenderNode {
    /// A reference to the DOM node that uses this style
    pub node: NodeRef,
    /// A property HashMap containing style property & value
    pub properties: Properties,
    /// Child style nodes
    pub children: Vec<RenderNode>,
}

impl RenderNode {
    pub fn get_style_value(&self, prop: Property) -> Value {
        // we will do defaulting here to reduce the size of properties map
        if let Some(value) = self.properties.get(&prop) {
            if let Some(v) = value {
                return v.clone();
            }
        }
        return Value::default(&prop);
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
            Property::Display => Display::default(),
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

/// Build the render tree using the root node & list of stylesheets
pub fn build_render_tree(node: NodeRef, rules: &[ContextualRule]) -> Option<RenderNode> {
    let properties = if node.is::<dom::text::Text>() {
        HashMap::new()
    } else {
        apply_styles(&node, &rules)
    };

    // Filter display none from render tree
    if let Some(display_value) = properties.get(&Property::Display) {
        if let Some(value) = display_value {
            if let Value::Display(Display::None) = value {
                return None;
            }
        }
    }

    Some(RenderNode {
        node: node.clone(),
        properties,
        children: node
            .borrow()
            .as_node()
            .child_nodes()
            .into_iter() // this is fine because we clone the node when iterate
            .filter_map(|child| build_render_tree(child, &rules))
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use crate::value_processing::{CSSLocation, CascadeOrigin};
    use css::cssom::css_rule::CSSRule;

    #[test]
    fn build_tree_simple() {
        let dom_tree = element(
            "div#parent",
            vec![element("div#child", vec![text("Hello")])],
        );

        let css = r#"
        #parent {
            background-color: white;
        }
        #parent #child {
            color: white;
        }
        #child {
            display: block;
        }
        "#;

        let stylesheet = parse_stylesheet(css);

        let rules = stylesheet
            .iter()
            .map(|rule| match rule {
                CSSRule::Style(style) => ContextualRule {
                    inner: style,
                    location: CSSLocation::Embedded,
                    origin: CascadeOrigin::User,
                },
            })
            .collect::<Vec<ContextualRule>>();

        let render_tree = build_render_tree(dom_tree.clone(), &rules)
            .expect("Render tree is not constructed");

        let mut parent_styles = render_tree.properties.values();
        assert_eq!(
            parent_styles.next(),
            Some(&Some(Value::Color(Color::RGBA(255, 255, 255, 255))))
        );

        let mut child_styles = render_tree.children[0].properties.values();
        assert_eq!(
            child_styles.next(),
            Some(&Some(Value::Color(Color::RGBA(255, 255, 255, 255))))
        );
        assert_eq!(
            child_styles.next(),
            Some(&Some(Value::Display(Display::Block)))
        );
    }
}
