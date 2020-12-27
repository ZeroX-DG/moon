use super::flow;

/// Formatting context of each box
#[derive(Debug, Clone, PartialEq)]
pub enum FormattingContext {
    Flow(flow::FormattingContext)
}