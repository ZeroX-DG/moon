use super::style_rule::StyleRule;

#[derive(Debug, PartialEq)]
pub enum CSSRule {
    Style(StyleRule),
}
