use css::cssom::stylesheet::StyleSheet;

pub struct CSSLoader;

impl CSSLoader {
    pub fn load_from_text(css: String) -> StyleSheet {
        let tokenizer = css::tokenizer::Tokenizer::new(css.chars());
        let mut parser = css::parser::Parser::<css::tokenizer::token::Token>::new(tokenizer.run());
        parser.parse_a_css_stylesheet()
    }
}
