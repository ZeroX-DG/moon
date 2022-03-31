use crate::property::Property;
use crate::value::Value;
use crate::value_processing::{compute, ComputeContext, Properties, StyleCache};
use crate::values::display::{Display, DisplayBox};
use dom::node::Node;
use strum::IntoEnumIterator;
use style_types::ContextualRule;

use super::inheritable::INHERITABLES;
use super::render_tree::{RenderNode, RenderTree};
use super::value_processing::{apply_styles, ValueRef};
use std::collections::HashMap;
use std::rc::{Rc, Weak};

pub struct TreeBuilder;

impl TreeBuilder {
    pub fn build(node: Rc<Node>, rules: &[ContextualRule]) -> RenderTree {
        let mut style_cache = StyleCache::new();
        let render_root = if node.is_document() {
            // the first child is HTML tag
            node.first_child()
        } else {
            Some(node)
        };

        let root = match render_root {
            Some(node) => build_from_node(node, rules, None, &mut style_cache),
            None => None,
        };

        RenderTree { root, style_cache }
    }
}

fn build_from_node(
    node: Rc<Node>,
    rules: &[ContextualRule],
    parent: Option<Weak<RenderNode>>,
    cache: &mut StyleCache,
) -> Option<Rc<RenderNode>> {
    let properties = if node.is_text() {
        HashMap::new()
    } else {
        apply_styles(&node, &rules)
    };

    // Filter display none from render tree
    if let Some(display_value) = properties.get(&Property::Display) {
        if let Some(value) = display_value {
            if let Value::Display(Display::Box(DisplayBox::None)) = value {
                return None;
            }
        }
    }

    let render_node = Rc::new(RenderNode {
        node: node.clone(),
        properties: compute_styles(properties, parent.clone(), cache),
        parent_render_node: parent,
        children: Default::default(),
    });

    render_node.children.replace(
        node.child_nodes()
            .into_iter() // this is fine because we clone the node when iterate
            .filter_map(|child| {
                build_from_node(child, &rules, Some(Rc::downgrade(&render_node)), cache)
            })
            .collect(),
    );

    Some(render_node)
}

fn compute_styles(
    properties: Properties,
    parent: Option<Weak<RenderNode>>,
    cache: &mut StyleCache,
) -> HashMap<Property, ValueRef> {
    // get inherit value for a property
    let inherit = |property: Property| {
        if let Some(parent) = &parent {
            if let Some(p) = parent.upgrade() {
                return (property.clone(), (**p.get_style(&property)).clone());
            }
        }
        // if there's no parent
        // we will use the initial value for that property
        return (property.clone(), Value::initial(&property));
    };

    // Step 3
    let specified_values = Property::iter()
        .map(|property| {
            if let Some(value) = properties.get(&property) {
                if let Some(v) = value {
                    // Explicit defaulting
                    // https://www.w3.org/TR/css3-cascade/#defaulting-keywords
                    if let Value::Initial = v {
                        return (property.clone(), Value::initial(&property));
                    }
                    if let Value::Inherit = v {
                        return inherit(property);
                    }
                    if let Value::Unset = v {
                        if INHERITABLES.contains(&property) {
                            return inherit(property);
                        }
                        return (property.clone(), Value::initial(&property));
                    }
                    return (property, v.clone());
                }
            }
            // if there's no specified value in properties
            // we will try to inherit it
            if INHERITABLES.contains(&property) {
                return inherit(property);
            }
            // if the property is not inheritable
            // we will use the initial value for that property
            return (property.clone(), Value::initial(&property));
        })
        .collect::<HashMap<Property, Value>>();

    // Step 4
    // TODO: Might be an expensive clone when we support all properties
    let temp_specified = specified_values.clone();
    let mut context = ComputeContext {
        parent,
        properties: temp_specified,
        style_cache: cache,
    };
    let computed_values = specified_values
        .into_iter()
        .map(|(property, value)| {
            let computed_value = compute(&property, &value, &mut context);
            return (property.clone(), computed_value);
        })
        .collect::<HashMap<Property, ValueRef>>();

    computed_values
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::border_style::BorderStyle;
    use crate::values::border_width::BorderWidth;
    use crate::values::color::Color;
    use crate::values::display::Display;
    use crate::values::length::{Length, LengthUnit};
    use crate::values::number::Number;
    use css::cssom::css_rule::CSSRule;
    use style_types::{CSSLocation, CascadeOrigin};
    use std::rc::Rc;
    use test_utils::css::parse_stylesheet;
    use test_utils::dom_creator::*;

    #[test]
    fn build_tree_simple() {
        let document = document();
        let dom_tree = element(
            "div#parent",
            document.clone(),
            vec![element(
                "div#child",
                document.clone(),
                vec![text("Hello", document.clone())],
            )],
        );

        let css = r#"
        #parent {
            background-color: white;
        }
        #parent #child {
            color: rgba(255, 255, 255, 255);
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

        let render_tree = TreeBuilder::build(dom_tree.clone(), &rules);

        let render_tree_inner = render_tree.root.expect("No root node");
        let parent_styles = &render_tree_inner.properties;
        assert_eq!(
            parent_styles.get(&Property::BackgroundColor),
            Some(&ValueRef(Rc::new(Value::Color(Color::Rgba(
                255.0.into(),
                255.0.into(),
                255.0.into(),
                255.0.into()
            )))))
        );

        let child_inner = render_tree_inner.children.borrow()[0].clone();
        let child_styles = &child_inner.properties;
        assert_eq!(
            child_styles.get(&Property::Color),
            Some(&ValueRef(Rc::new(Value::Color(Color::Rgba(
                255.0.into(),
                255.0.into(),
                255.0.into(),
                255.0.into()
            )))))
        );
        assert_eq!(
            child_styles.get(&Property::Display),
            Some(&ValueRef(Rc::new(Value::Display(Display::new_block()))))
        );
    }

    #[test]
    fn shorthand_property() {
        let document = document();
        let dom_tree = element("div#parent", document.clone(), vec![]);

        let css = r#"
        #parent {
            margin: 20px;
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

        let render_tree = TreeBuilder::build(dom_tree.clone(), &rules);

        let render_tree_inner = render_tree.root.expect("No root node");
        let parent_styles = &render_tree_inner.properties;
        assert_eq!(
            parent_styles.get(&Property::MarginTop),
            Some(&ValueRef(Rc::new(Value::Length(Length {
                value: Number(20.0),
                unit: LengthUnit::Px
            }))))
        );
        assert_eq!(
            parent_styles.get(&Property::MarginRight),
            Some(&ValueRef(Rc::new(Value::Length(Length {
                value: Number(20.0),
                unit: LengthUnit::Px
            }))))
        );
        assert_eq!(
            parent_styles.get(&Property::MarginBottom),
            Some(&ValueRef(Rc::new(Value::Length(Length {
                value: Number(20.0),
                unit: LengthUnit::Px
            }))))
        );
        assert_eq!(
            parent_styles.get(&Property::MarginLeft),
            Some(&ValueRef(Rc::new(Value::Length(Length {
                value: Number(20.0),
                unit: LengthUnit::Px
            }))))
        );
    }

    #[test]
    fn invalid_shorthand() {
        let dom_tree = element("div#parent", document(), vec![]);

        let css = r#"
        #parent {
            border: 2px solid black red;
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

        let render_tree = TreeBuilder::build(dom_tree.clone(), &rules);

        let render_tree_inner = render_tree.root.expect("No root node");
        let parent_styles = &render_tree_inner.properties;
        assert_eq!(
            parent_styles.get(&Property::BorderTopColor),
            Some(&ValueRef(Rc::new(Value::Color(Color::black()))))
        );
        assert_eq!(
            parent_styles.get(&Property::BorderRightColor),
            Some(&ValueRef(Rc::new(Value::Color(Color::black()))))
        );
        assert_eq!(
            parent_styles.get(&Property::BorderBottomColor),
            Some(&ValueRef(Rc::new(Value::Color(Color::black()))))
        );
        assert_eq!(
            parent_styles.get(&Property::BorderLeftColor),
            Some(&ValueRef(Rc::new(Value::Color(Color::black()))))
        );
        assert_eq!(
            parent_styles.get(&Property::BorderTopWidth),
            Some(&ValueRef(Rc::new(Value::BorderWidth(BorderWidth::Medium))))
        );
        assert_eq!(
            parent_styles.get(&Property::BorderRightWidth),
            Some(&ValueRef(Rc::new(Value::BorderWidth(BorderWidth::Medium))))
        );
        assert_eq!(
            parent_styles.get(&Property::BorderBottomWidth),
            Some(&ValueRef(Rc::new(Value::BorderWidth(BorderWidth::Medium))))
        );
        assert_eq!(
            parent_styles.get(&Property::BorderLeftWidth),
            Some(&ValueRef(Rc::new(Value::BorderWidth(BorderWidth::Medium))))
        );
        assert_eq!(
            parent_styles.get(&Property::BorderTopStyle),
            Some(&ValueRef(Rc::new(Value::BorderStyle(BorderStyle::None))))
        );
        assert_eq!(
            parent_styles.get(&Property::BorderRightStyle),
            Some(&ValueRef(Rc::new(Value::BorderStyle(BorderStyle::None))))
        );
        assert_eq!(
            parent_styles.get(&Property::BorderBottomStyle),
            Some(&ValueRef(Rc::new(Value::BorderStyle(BorderStyle::None))))
        );
        assert_eq!(
            parent_styles.get(&Property::BorderLeftStyle),
            Some(&ValueRef(Rc::new(Value::BorderStyle(BorderStyle::None))))
        );
    }

    #[test]
    fn shorthand_property_3_values() {
        let dom_tree = element("div#parent", document(), vec![]);

        let css = r#"
        #parent {
            padding: 20px 10px 20px;
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

        let render_tree = TreeBuilder::build(dom_tree.clone(), &rules);

        let render_tree_inner = render_tree.root.expect("No root node");
        let parent_styles = &render_tree_inner.properties;
        assert_eq!(
            parent_styles.get(&Property::PaddingTop),
            Some(&ValueRef(Rc::new(Value::Length(Length {
                value: Number(20.0),
                unit: LengthUnit::Px
            }))))
        );
        assert_eq!(
            parent_styles.get(&Property::PaddingRight),
            Some(&ValueRef(Rc::new(Value::Length(Length {
                value: Number(10.0),
                unit: LengthUnit::Px
            }))))
        );
        assert_eq!(
            parent_styles.get(&Property::PaddingBottom),
            Some(&ValueRef(Rc::new(Value::Length(Length {
                value: Number(20.0),
                unit: LengthUnit::Px
            }))))
        );
        assert_eq!(
            parent_styles.get(&Property::PaddingLeft),
            Some(&ValueRef(Rc::new(Value::Length(Length {
                value: Number(0.0),
                unit: LengthUnit::Px
            }))))
        );
    }

    #[test]
    fn explicit_default() {
        let document = document();
        let dom_tree = element(
            "div#parent",
            document.clone(),
            vec![element("div#child", document.clone(), vec![])],
        );

        let css = r#"
        #parent {
            color: red;
        }
        #parent #child {
            color: initial;
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

        let render_tree = TreeBuilder::build(dom_tree.clone(), &rules);

        let render_tree_inner = render_tree.root.expect("No root node");
        let parent_styles = &render_tree_inner.properties;
        assert_eq!(
            parent_styles.get(&Property::Color),
            Some(&ValueRef(Rc::new(Value::Color(Color::Rgba(
                Number(255.0),
                Number(0.0),
                Number(0.0),
                Number(255.0),
            )))))
        );

        let child_inner = render_tree_inner.children.borrow()[0].clone();
        let child_styles = &child_inner.properties;

        assert_eq!(
            child_styles.get(&Property::Color),
            Some(&ValueRef(Rc::new(Value::Color(Color::black()))))
        );
    }

    #[test]
    fn shorthand_property_special() {
        let dom_tree = element("div#parent", document(), vec![]);

        let css = r#"
        #parent {
            margin: 20px 10px;
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

        let render_tree = TreeBuilder::build(dom_tree.clone(), &rules);

        let render_tree_inner = render_tree.root.expect("No root node");
        let parent_styles = &render_tree_inner.properties;
        assert_eq!(
            parent_styles.get(&Property::MarginTop),
            Some(&ValueRef(Rc::new(Value::Length(Length {
                value: Number(20.0),
                unit: LengthUnit::Px
            }))))
        );
        assert_eq!(
            parent_styles.get(&Property::MarginRight),
            Some(&ValueRef(Rc::new(Value::Length(Length {
                value: Number(10.0),
                unit: LengthUnit::Px
            }))))
        );
        assert_eq!(
            parent_styles.get(&Property::MarginBottom),
            Some(&ValueRef(Rc::new(Value::Length(Length {
                value: Number(20.0),
                unit: LengthUnit::Px
            }))))
        );
        assert_eq!(
            parent_styles.get(&Property::MarginLeft),
            Some(&ValueRef(Rc::new(Value::Length(Length {
                value: Number(10.0),
                unit: LengthUnit::Px
            }))))
        );
    }

    #[test]
    fn shorthand_property_border() {
        let dom_tree = element("div#parent", document(), vec![]);

        let css = r#"
        #parent {
            border: dotted 2px;
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

        let render_tree = TreeBuilder::build(dom_tree.clone(), &rules);

        let render_tree_inner = render_tree.root.expect("No root node");
        let parent_styles = &render_tree_inner.properties;
        assert_eq!(
            parent_styles.get(&Property::BorderTopColor),
            Some(&ValueRef(Rc::new(Value::Color(Color::black()))))
        );
        assert_eq!(
            parent_styles.get(&Property::BorderRightColor),
            Some(&ValueRef(Rc::new(Value::Color(Color::black()))))
        );
        assert_eq!(
            parent_styles.get(&Property::BorderBottomColor),
            Some(&ValueRef(Rc::new(Value::Color(Color::black()))))
        );
        assert_eq!(
            parent_styles.get(&Property::BorderLeftColor),
            Some(&ValueRef(Rc::new(Value::Color(Color::black()))))
        );
        assert_eq!(
            parent_styles.get(&Property::BorderTopWidth),
            Some(&ValueRef(Rc::new(Value::Length(Length {
                value: Number(2.0),
                unit: LengthUnit::Px
            }))))
        );
        assert_eq!(
            parent_styles.get(&Property::BorderRightWidth),
            Some(&ValueRef(Rc::new(Value::Length(Length {
                value: Number(2.0),
                unit: LengthUnit::Px
            }))))
        );
        assert_eq!(
            parent_styles.get(&Property::BorderBottomWidth),
            Some(&ValueRef(Rc::new(Value::Length(Length {
                value: Number(2.0),
                unit: LengthUnit::Px
            }))))
        );
        assert_eq!(
            parent_styles.get(&Property::BorderLeftWidth),
            Some(&ValueRef(Rc::new(Value::Length(Length {
                value: Number(2.0),
                unit: LengthUnit::Px
            }))))
        );
        assert_eq!(
            parent_styles.get(&Property::BorderTopStyle),
            Some(&ValueRef(Rc::new(Value::BorderStyle(BorderStyle::Dotted))))
        );
        assert_eq!(
            parent_styles.get(&Property::BorderRightStyle),
            Some(&ValueRef(Rc::new(Value::BorderStyle(BorderStyle::Dotted))))
        );
        assert_eq!(
            parent_styles.get(&Property::BorderBottomStyle),
            Some(&ValueRef(Rc::new(Value::BorderStyle(BorderStyle::Dotted))))
        );
        assert_eq!(
            parent_styles.get(&Property::BorderLeftStyle),
            Some(&ValueRef(Rc::new(Value::BorderStyle(BorderStyle::Dotted))))
        );
    }
}
