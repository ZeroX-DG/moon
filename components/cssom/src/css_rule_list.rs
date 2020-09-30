use super::css_rule::CSSRule;

pub struct CSSRuleList(Vec<CSSRule>);

impl CSSRuleList {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl From<Vec<CSSRule>> for CSSRuleList {
    fn from(rules: Vec<CSSRule>) -> Self {
        Self(rules)
    }
}
