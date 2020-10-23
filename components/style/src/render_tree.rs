use super::inheritable::INHERITABLES;
use super::value_processing::{
    apply_styles, compute, ComputeContext, ContextualRule, Properties, Property, Value, ValueRef,
};
use super::values::display::Display;
use dom::dom_ref::NodeRef;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};
use strum::IntoEnumIterator;

pub type RenderNodeRef = Rc<RefCell<RenderNode>>;
pub type RenderNodeWeak = Weak<RefCell<RenderNode>>;

pub struct RenderTree {
    /// The root node of the render tree
    pub root: Option<RenderNodeRef>,
    /// The style cache to share style value and reduce style size
    pub style_cache: HashSet<ValueRef>,
}

/// A style node in the style tree
#[derive(Debug)]
pub struct RenderNode {
    /// A reference to the DOM node that uses this style
    pub node: NodeRef,
    /// A property HashMap containing computed styles
    pub properties: HashMap<Property, ValueRef>,
    /// Child style nodes
    pub children: Vec<RenderNodeRef>,
    /// Parent reference for inheritance
    pub parent_render_node: Option<RenderNodeWeak>,
}

impl RenderNode {
    /// Get style value of a property
    pub fn get_style(&self, property: &Property) -> ValueRef {
        if let Some(value) = self.properties.get(property) {
            return value.clone();
        }

        if INHERITABLES.contains(property) {
            if let Some(parent) = &self.parent_render_node {
                if let Some(p) = parent.upgrade() {
                    return p.borrow().get_style(&property);
                }
            }
        }

        panic!("Oops, we should not reach here");
    }
}

pub fn compute_styles(
    properties: Properties,
    parent: Option<RenderNodeWeak>,
    cache: &mut HashSet<ValueRef>,
) -> HashMap<Property, ValueRef> {
    // Step 3
    let specified_values = Property::iter()
        .map(|property| {
            if let Some(value) = properties.get(&property) {
                // TODO: explicit defaulting
                if let Some(v) = value {
                    return (property, v.clone());
                }
            }
            // if there's no specified value in properties
            // we will try to inherit it
            if INHERITABLES.contains(&property) {
                if let Some(parent) = &parent {
                    if let Some(p) = parent.upgrade() {
                        return (
                            property.clone(),
                            (**p.borrow().get_style(&property)).clone(),
                        );
                    }
                }
            }
            // if there's no parent or the property is not inheritable
            // we will use the initial value for that property
            return (property.clone(), Value::initial(&property));
        })
        .collect::<HashMap<Property, Value>>();

    // Step 4
    // TODO: Might be an expensive clone when we support all properties
    let temp_specified = specified_values.clone();
    let mut context = ComputeContext {
        parent: &parent,
        properties: temp_specified,
        style_cache: cache,
    };
    let computed_values = specified_values
        .into_iter()
        .map(|(property, value)| {
            // TODO: filter properties that need layout to compute
            let computed_value = compute(&property, &value, &mut context);
            return (property.clone(), computed_value);
        })
        .collect::<HashMap<Property, ValueRef>>();

    computed_values
}

pub fn build_render_tree(node: NodeRef, rules: &[ContextualRule]) -> RenderTree {
    let mut style_cache = HashSet::new();
    let root = build_render_tree_from_node(node, rules, None, &mut style_cache);
    RenderTree { root, style_cache }
}

/// Build the render tree using the root node & list of stylesheets
fn build_render_tree_from_node(
    node: NodeRef,
    rules: &[ContextualRule],
    parent: Option<RenderNodeWeak>,
    cache: &mut HashSet<ValueRef>,
) -> Option<RenderNodeRef> {
    let properties = if node.is::<dom::text::Text>() {
        HashMap::new()
    } else {
        apply_styles(&node, &rules)
    };

    // Filter head from render tree
    if let Some(element) = node.borrow().as_element() {
        if element.tag_name() == "head" {
            return None;
        }
    }

    // Filter display none from render tree
    if let Some(display_value) = properties.get(&Property::Display) {
        if let Some(value) = display_value {
            if let Value::Display(Display::None) = value {
                return None;
            }
        }
    }

    let render_node = Rc::new(RefCell::new(RenderNode {
        node: node.clone(),
        properties: compute_styles(properties, parent.clone(), cache),
        parent_render_node: parent,
        children: Vec::new(),
    }));

    render_node.borrow_mut().children = node
        .borrow()
        .as_node()
        .child_nodes()
        .into_iter() // this is fine because we clone the node when iterate
        .filter_map(|child| {
            build_render_tree_from_node(child, &rules, Some(Rc::downgrade(&render_node)), cache)
        })
        .collect();

    Some(render_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use crate::value_processing::{CSSLocation, CascadeOrigin};
    use crate::values::color::Color;
    use crate::values::display::Display;
    use crate::values::length::{Length, LengthUnit};
    use crate::values::number::Number;
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

        let render_tree = build_render_tree(dom_tree.clone(), &rules);

        let render_tree_inner = render_tree.root.expect("No root node");
        let render_tree_inner = render_tree_inner.borrow();
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

        let child_inner = render_tree_inner.children[0].borrow();
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
            Some(&ValueRef(Rc::new(Value::Display(Display::Block))))
        );
    }

    #[test]
    fn shorthand_property() {
        let dom_tree = element(
            "div#parent",
            vec![]
        );

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

        let render_tree = build_render_tree(dom_tree.clone(), &rules);

        let render_tree_inner = render_tree.root.expect("No root node");
        let render_tree_inner = render_tree_inner.borrow();
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
}