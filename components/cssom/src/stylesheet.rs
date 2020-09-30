use super::css_rule_list::CSSRuleList;
use super::css_rule::CSSRule;

pub struct StyleSheet {
    css_rules: CSSRuleList
}

impl StyleSheet {
    pub fn new() -> Self {
        Self {
            css_rules: CSSRuleList::new()
        }
    }
}

impl From<Vec<CSSRule>> for StyleSheet {
    fn from(rules: Vec<CSSRule>) -> Self {
        Self {
            css_rules: CSSRuleList::from(rules)
        }
    }
}
