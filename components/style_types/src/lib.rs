use css::cssom::style_rule::StyleRule;

/// Location of the CSS applied
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CSSLocation {
    /// Inline CSS (in HTML tags)
    Inline,
    /// Embedded CSS (in HTML style tag)
    Embedded,
    /// External CSS (in external css file)
    External,
}

/// Cascade origin
/// https://www.w3.org/TR/css-cascade-4/#origin
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CascadeOrigin {
    Author,
    User,
    UserAgent,
}

/// Style rule with context for cascading
#[derive(Debug)]
pub struct ContextualRule<'a> {
    pub inner: &'a StyleRule,
    pub origin: CascadeOrigin,
    pub location: CSSLocation,
}