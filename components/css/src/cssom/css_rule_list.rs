use super::css_rule::CSSRule;
use std::ops::Deref;

#[derive(Debug, PartialEq)]
pub struct CSSRuleList(pub Vec<CSSRule>);

impl CSSRuleList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn append_rule(&mut self, rule: CSSRule) {
        self.0.push(rule);
    }
}

impl Deref for CSSRuleList {
    type Target = Vec<CSSRule>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
