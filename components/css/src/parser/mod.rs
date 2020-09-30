use super::tokenizer::Tokenizer;
use super::tokenizer::token::Token;
use cssom::stylesheet::StyleSheet;
use cssom::css_rule::CSSRule;

pub struct Parser {
    /// Tokenizer to receive CSS token
    tokenizer: Tokenizer,
    /// Top level flag
    top_level: bool,
    /// Reconsume current input token
    reconsume: bool,
    /// Current token to return if being reconsumed
    current_token: Option<Token>
}

impl Parser {
    pub fn new(tokenizer: Tokenizer) -> Self {
        Self {
            tokenizer,
            top_level: false,
            reconsume: false,
            current_token: None
        }
    }

    fn consume_next_token(&mut self) -> Token {
        if self.reconsume {
            self.reconsume = false;
            return self.current_token.clone().unwrap();
        }
        let token = self.tokenizer.consume_token();
        self.current_token = Some(token.clone());
        return token;
    }

    fn reconsume(&mut self) {
        self.reconsume = true;
    }

    pub fn consume_a_qualified_rule(&mut self) -> Option<CSSRule> {
        return None
    }

    pub fn consume_rules(&mut self) -> Vec<CSSRule> {
        let mut rules = Vec::new();
        loop {
            let next_token = self.consume_next_token();
            
            match next_token {
                Token::Whitespace => continue,
                Token::EOF => return rules,
                Token::CDO | Token::CDC => {
                    if self.top_level {
                        continue;
                    }
                    self.reconsume();
                    if let Some(rule) = self.consume_a_qualified_rule() {
                        rules.push(rule);
                    }
                }
                // TODO: impl support for @ rules
                _ => {
                    self.reconsume();
                    if let Some(rule) = self.consume_a_qualified_rule() {
                        rules.push(rule);
                    }
                }
            }
        }
    }
}

impl Parser {
    pub fn parse_stylesheet(&mut self) -> StyleSheet {
        self.top_level = true;
        let rules = self.consume_rules();
        return StyleSheet::from(rules);
    }
}
