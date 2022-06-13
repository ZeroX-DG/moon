use css::selector::structs::*;
use dom::{element::Element, node::NodePtr};

fn get_parent(el: &NodePtr) -> Option<NodePtr> {
    let parent = el.parent();
    if let Some(p) = parent {
        if p.is_element() {
            return Some(NodePtr(p));
        }
    }
    None
}

fn get_prev_sibling(el: &NodePtr) -> Option<NodePtr> {
    el.prev_sibling().map(|node| NodePtr(node))
}

pub fn is_match_selectors(element: &NodePtr, selectors: &Vec<Selector>) -> bool {
    selectors
        .iter()
        .any(|selector| is_match_selector(element.clone(), selector))
}

pub fn is_match_selector(element: NodePtr, selector: &Selector) -> bool {
    let mut current_element = Some(element);
    for (selector_seq, combinator) in selector.values().iter().rev() {
        if let Some(el) = current_element.clone() {
            match combinator {
                Some(Combinator::Child) => {
                    let parent = get_parent(&el);
                    if let Some(p) = &parent {
                        if !is_match_simple_selector_seq(p, selector_seq) {
                            return false;
                        }
                    }
                    current_element = parent;
                }
                Some(Combinator::Descendant) => loop {
                    let parent = get_parent(&el);
                    if let Some(p) = &parent {
                        if is_match_simple_selector_seq(p, selector_seq) {
                            current_element = parent;
                            break;
                        }
                    }
                    return false;
                },
                Some(Combinator::NextSibling) => {
                    let sibling = get_prev_sibling(&el);
                    if let Some(sibling) = &sibling {
                        if !is_match_simple_selector_seq(sibling, selector_seq) {
                            return false;
                        }
                    }
                    current_element = sibling;
                }
                Some(Combinator::SubsequentSibling) => loop {
                    let sibling = get_prev_sibling(&el);
                    if let Some(s) = &sibling {
                        if is_match_simple_selector_seq(s, selector_seq) {
                            current_element = sibling;
                            break;
                        }
                    }
                    return false;
                },
                None => {
                    if !is_match_simple_selector_seq(&el, selector_seq) {
                        return false;
                    }
                }
            }
        } else {
            return false;
        }
    }
    true
}

fn is_match_simple_selector_seq(element: &NodePtr, sequence: &SimpleSelectorSequence) -> bool {
    if let Some(element) = element.as_element_opt() {
        return sequence
            .values()
            .iter()
            .all(|selector| is_match_simple_selector(element, selector));
    }

    log::warn!("Trying to match selector on non-element: {:?}", element);
    false
}

fn is_match_simple_selector(element: &Element, selector: &SimpleSelector) -> bool {
    match selector.selector_type() {
        SimpleSelectorType::Universal => true,
        SimpleSelectorType::Type => {
            if let Some(type_name) = selector.value() {
                return element.tag_name() == *type_name;
            }
            false
        }
        SimpleSelectorType::Class => {
            if let Some(type_name) = selector.value() {
                return element.class_list().borrow().contains(&type_name);
            }
            false
        }
        SimpleSelectorType::ID => {
            if let Some(id) = selector.value() {
                return element.id().map(|value| value == *id).unwrap_or(false);
            }
            false
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use css::cssom::css_rule::CSSRule;
    use css::parser::Parser;
    use css::tokenizer::token::Token;
    use css::tokenizer::Tokenizer;
    use dom::create_element;
    use shared::tree_node::WeakTreeNode;
    use test_utils::dom_creator::document;

    #[test]
    fn match_simple_type() {
        let element = create_element(WeakTreeNode::from(&document().0), "h1");
        let css = "h1 { color: red; }";

        let tokenizer = Tokenizer::new(css.chars());
        let tokens = tokenizer.run();
        let mut parser = Parser::<Token>::new(tokens);
        let stylesheet = parser.parse_a_css_stylesheet();

        let rule = stylesheet.first().unwrap();

        match rule {
            CSSRule::Style(style) => {
                let selectors = &style.selectors;
                assert!(is_match_selectors(&element, selectors));
            }
        }
    }

    #[test]
    fn match_simple_id() {
        let element_node = create_element(WeakTreeNode::from(&document().0), "h1");
        element_node.as_element().set_attribute("id", "button");
        let css = "h1#button { color: red; }";

        let tokenizer = Tokenizer::new(css.chars());
        let tokens = tokenizer.run();
        let mut parser = Parser::<Token>::new(tokens);
        let stylesheet = parser.parse_a_css_stylesheet();

        let rule = stylesheet.first().unwrap();

        match rule {
            CSSRule::Style(style) => {
                let selectors = &style.selectors;
                assert!(is_match_selectors(&element_node, selectors));
            }
        }
    }

    #[test]
    fn match_simple_decendant() {
        let doc = document();
        let parent = create_element(WeakTreeNode::from(&doc.0), "h1");
        let child = create_element(WeakTreeNode::from(&doc.0), "button");
        parent.append_child(child.0.clone());

        let css = "h1 button { color: red; }";

        let tokenizer = Tokenizer::new(css.chars());
        let tokens = tokenizer.run();
        let mut parser = Parser::<Token>::new(tokens);
        let stylesheet = parser.parse_a_css_stylesheet();

        let rule = stylesheet.first().unwrap();

        match rule {
            CSSRule::Style(style) => {
                let selectors = &style.selectors;
                assert!(is_match_selectors(&child, selectors));
            }
        }
    }

    #[test]
    fn match_simple_child() {
        let doc = document();
        let parent = create_element(WeakTreeNode::from(&doc.0), "h1");
        let child = create_element(WeakTreeNode::from(&doc.0), "button");
        parent.append_child(child.0.clone());

        let css = "h1 > button { color: red; }";

        let tokenizer = Tokenizer::new(css.chars());
        let tokens = tokenizer.run();
        let mut parser = Parser::<Token>::new(tokens);
        let stylesheet = parser.parse_a_css_stylesheet();

        let rule = stylesheet.first().unwrap();

        match rule {
            CSSRule::Style(style) => {
                let selectors = &style.selectors;
                assert!(is_match_selectors(&child, selectors));
            }
        }
    }

    #[test]
    fn match_invalid_child() {
        let doc = document();
        let parent = create_element(WeakTreeNode::from(&doc.0), "h1");
        let child = create_element(WeakTreeNode::from(&doc.0), "button");
        parent.append_child(child.0.clone());

        let css = "button > h1 { color: red; }";

        let tokenizer = Tokenizer::new(css.chars());
        let tokens = tokenizer.run();
        let mut parser = Parser::<Token>::new(tokens);
        let stylesheet = parser.parse_a_css_stylesheet();

        let rule = stylesheet.first().unwrap();

        match rule {
            CSSRule::Style(style) => {
                let selectors = &style.selectors;
                assert!(!is_match_selectors(&child, selectors));
            }
        }
    }

    #[test]
    fn match_invalid_id() {
        let doc = document();
        let parent = create_element(WeakTreeNode::from(&doc.0), "h1");
        let child = create_element(WeakTreeNode::from(&doc.0), "button");
        parent.append_child(child.0.clone());

        let css = "h1#name > button { color: red; }";

        let tokenizer = Tokenizer::new(css.chars());
        let tokens = tokenizer.run();
        let mut parser = Parser::<Token>::new(tokens);
        let stylesheet = parser.parse_a_css_stylesheet();

        let rule = stylesheet.first().unwrap();

        match rule {
            CSSRule::Style(style) => {
                let selectors = &style.selectors;
                assert!(!is_match_selectors(&child, selectors));
            }
        }
    }

    #[test]
    fn match_group_of_types() {
        let doc = document();
        let parent = create_element(WeakTreeNode::from(&doc.0), "h1");
        let child = create_element(WeakTreeNode::from(&doc.0), "button");
        parent.append_child(child.0.clone());

        let css = "h1, button { color: red; }";

        let tokenizer = Tokenizer::new(css.chars());
        let tokens = tokenizer.run();
        let mut parser = Parser::<Token>::new(tokens);
        let stylesheet = parser.parse_a_css_stylesheet();

        let rule = stylesheet.first().unwrap();

        match rule {
            CSSRule::Style(style) => {
                let selectors = &style.selectors;
                assert!(is_match_selectors(&child, selectors));
                assert!(is_match_selectors(&parent, selectors));
            }
        }
    }
}
