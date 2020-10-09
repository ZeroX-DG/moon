use super::style_rule::StyleRule;

#[derive(Debug)]
pub enum CSSRule {
    Style(StyleRule),
}
