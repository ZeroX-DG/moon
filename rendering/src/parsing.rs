use css::cssom::stylesheet::StyleSheet;
use dom::dom_ref::NodeRef;

pub fn parse_html(html: String) -> NodeRef {
    let tokenizer = html::tokenizer::Tokenizer::new(html.chars());
    let tree_builder = html::tree_builder::TreeBuilder::new(tokenizer);
    tree_builder.run()
}

pub fn parse_css(css: String) -> StyleSheet {
    let tokenizer = css::tokenizer::Tokenizer::new(css.chars());
    let mut parser = css::parser::Parser::<css::tokenizer::token::Token>::new(tokenizer.run());
    parser.parse_a_css_stylesheet()
}
