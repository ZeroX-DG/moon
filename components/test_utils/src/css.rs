use css::cssom::stylesheet::StyleSheet;
use css::parser::Parser;
use css::tokenizer::token::Token;
use css::tokenizer::Tokenizer;

pub fn parse_stylesheet(style: &str) -> StyleSheet {
    let tokenizer = Tokenizer::new(style.chars());
    let mut parser = Parser::<Token>::new(tokenizer.run());
    parser.parse_a_css_stylesheet()
}
