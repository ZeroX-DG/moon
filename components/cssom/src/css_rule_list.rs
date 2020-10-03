use super::css_rule::CSSRule;

pub struct CSSRuleList(Vec<CSSRule>);

impl CSSRuleList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn append_rule(&mut self, rule: CSSRule) {
        self.0.push(rule);
    }
}
