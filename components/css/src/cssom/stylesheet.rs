use super::css_rule::CSSRule;
use super::css_rule_list::CSSRuleList;
use std::ops::Deref;

#[derive(Debug, PartialEq)]
pub struct StyleSheet {
    pub css_rules: CSSRuleList,
}

impl StyleSheet {
    pub fn new() -> Self {
        Self {
            css_rules: CSSRuleList::new(),
        }
    }

    pub fn append_rule(&mut self, rule: CSSRule) {
        self.css_rules.append_rule(rule);
    }
}

impl Deref for StyleSheet {
    type Target = CSSRuleList;

    fn deref(&self) -> &Self::Target {
        &self.css_rules
    }
}
