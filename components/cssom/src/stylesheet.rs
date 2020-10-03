use super::css_rule::CSSRule;
use super::css_rule_list::CSSRuleList;

pub struct StyleSheet {
    css_rules: CSSRuleList,
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
