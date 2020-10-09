pub mod structs;

use super::tokenizer::token::Token;
use super::cssom::stylesheet::StyleSheet;
use super::cssom::style_rule::StyleRule;
use super::cssom::css_rule::CSSRule;
use super::selector::parse_selectors;
use io::data_stream::DataStream;
use std::env;
use structs::*;

fn is_trace() -> bool {
    match env::var("TRACE_CSS_PARSER") {
        Ok(s) => s == "true" || s == "",
        _ => false,
    }
}

macro_rules! trace {
    ($err:expr) => {
        println!("[ParseError][CSS Parsing]: {}", $err);
    };
}

macro_rules! emit_error {
    ($err:expr) => {
        if is_trace() {
            trace!($err)
        }
    };
}

pub struct SyntaxError;

/// CSS Parser
pub struct Parser {
    /// Stream of output tokens from tokenizer
    tokens: DataStream<Token>,
    /// Top level flag
    top_level: bool,
    /// Reconsume current input token
    reconsume: bool,
    /// Current token to return if being reconsumed
    current_token: Option<Token>,
}

impl Parser {
    pub fn new(tokens: DataStream<Token>) -> Self {
        Self {
            tokens,
            top_level: false,
            reconsume: false,
            current_token: None,
        }
    }

    pub fn recreate(&mut self, tokens: DataStream<Token>) {
        self.top_level = false;
        self.reconsume = false;
        self.current_token = None;
        self.tokens = tokens;
    }

    fn consume_next_token(&mut self) -> Token {
        if self.reconsume {
            self.reconsume = false;
            return self.current_token.clone().unwrap();
        }
        let token = self.tokens.next().unwrap_or(&Token::EOF);
        self.current_token = Some(token.clone());
        return token.clone();
    }

    fn peek_next_token(&mut self) -> Token {
        if self.reconsume {
            return self.current_token.clone().unwrap();
        }
        return self.tokens.peek().unwrap_or(&Token::EOF).clone();
    }

    fn reconsume(&mut self) {
        self.reconsume = true;
    }

    fn ending_token(&self) -> Token {
        match self.current_token {
            Some(Token::BracketOpen) => Token::BracketClose,
            Some(Token::BraceOpen) => Token::BraceClose,
            Some(Token::ParentheseOpen) => Token::ParentheseClose,
            _ => panic!("Can't get ending token"),
        }
    }

    fn consume_a_qualified_rule(&mut self) -> Option<QualifiedRule> {
        let mut qualified_rule = QualifiedRule::new();

        loop {
            let next_token = self.consume_next_token();

            if let Token::EOF = next_token {
                emit_error!("Unexpected EOF while consuming a qualified rule");
                return None;
            }

            if let Token::BraceOpen = next_token {
                qualified_rule.set_block(self.consume_a_simple_block());
                return Some(qualified_rule);
            }

            // TODO: What is simple block with an associated token of <{-token>? How is it a token?

            self.reconsume();
            qualified_rule.append_prelude(self.consume_a_component_value());
        }
    }

    fn consume_a_component_value(&mut self) -> ComponentValue {
        let token = self.consume_next_token();

        match token {
            Token::BraceOpen | Token::BracketOpen | Token::ParentheseOpen => {
                return ComponentValue::SimpleBlock(self.consume_a_simple_block());
            }
            Token::Function(_) => {
                return ComponentValue::Function(self.consume_a_function());
            }
            t => ComponentValue::PerservedToken(t)
        }
    }

    fn consume_a_list_of_declarations(&mut self) -> Vec<DeclarationOrAtRule> {
        let mut result = Vec::new();

        loop {
            let next_token = self.consume_next_token();

            match next_token {
                Token::Whitespace | Token::Semicolon => {}
                Token::EOF => {
                    return result;
                }
                Token::AtKeyword(_) => {
                    self.reconsume();
                    let rule = self.consume_an_at_rule();
                    result.push(DeclarationOrAtRule::AtRule(rule));
                }
                Token::Ident(_) => {
                    let mut tmp = vec![self.current_token.clone().unwrap()];
                    loop {
                        match self.peek_next_token() {
                            Token::Semicolon | Token::EOF => break,
                            _ => {
                                if let ComponentValue::PerservedToken(t) = self.consume_a_component_value() {
                                    tmp.push(t);
                                }
                            }
                        }
                    }
                    let mut parser = Parser::new(DataStream::new(tmp));
                    if let Some(declaration) = parser.consume_a_declaration()
                    {
                        result.push(DeclarationOrAtRule::Declaration(declaration));
                    }
                }
                _ => {
                    emit_error!("Unexpected token while consuming a list of declarations");
                    self.reconsume();
                    loop {
                        match self.peek_next_token() {
                            Token::Semicolon | Token::EOF => break,
                            _ => {
                                // throw away
                                self.consume_a_component_value();
                            }
                        }
                    }
                }
            }
        }
    }

    fn consume_a_function(&mut self) -> Function {
        let current_token = self.current_token.clone().unwrap();
        let function_name = if let Token::Function(name) = current_token {
            name
        } else {
            panic!("The current token is not a function");
        };

        let mut function = Function::new(function_name);

        loop {
            let next_token = self.consume_next_token();

            match next_token {
                Token::ParentheseClose => return function,
                Token::EOF => {
                    emit_error!("Unexpected EOF while consuming a function");
                    return function;
                }
                _ => {
                    self.reconsume();
                    function.append_value(self.consume_a_component_value());
                }
            }
        }
    }

    fn consume_a_simple_block(&mut self) -> SimpleBlock {
        let ending_token = self.ending_token();
        let mut simple_block = SimpleBlock::new(self.current_token.clone().unwrap());

        loop {
            let next_token = self.consume_next_token();

            if next_token == ending_token {
                return simple_block;
            }

            if let Token::EOF = next_token {
                emit_error!("Unexpected EOF while consuming a simple block");
                return simple_block;
            }

            self.reconsume();
            simple_block.append_value(self.consume_a_component_value());
        }
    }

    fn consume_an_at_rule(&mut self) -> AtRule {
        self.consume_next_token();
        let current_token = self.current_token.clone().unwrap();
        let keyword_name = if let Token::AtKeyword(name) = current_token {
            name
        } else {
            panic!("The current token is not a function");
        };
        let mut at_rule = AtRule::new(keyword_name);

        loop {
            let next_token = self.consume_next_token();

            match next_token {
                Token::Semicolon => return at_rule,
                Token::EOF => {
                    emit_error!("Unexpected EOF while consuming an at-rule");
                    return at_rule;
                }
                Token::BraceOpen => {
                    at_rule.set_block(self.consume_a_simple_block());
                    return at_rule;
                }
                // TODO: How is a simple block a token?
                _ => {
                    self.reconsume();
                    at_rule.append_prelude(self.consume_a_component_value());
                }
            }
        }
    }

    fn consume_a_list_of_rules(&mut self) -> ListOfRules {
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
                        rules.push(Rule::QualifiedRule(rule));
                    }
                }
                Token::AtKeyword(_) => {
                    self.reconsume();
                    let at_rule = self.consume_an_at_rule();
                    rules.push(Rule::AtRule(at_rule));
                }
                _ => {
                    self.reconsume();
                    if let Some(rule) = self.consume_a_qualified_rule() {
                        rules.push(Rule::QualifiedRule(rule));
                    }
                }
            }
        }
    }

    fn consume_a_declaration(&mut self) -> Option<Declaration> {
        let next_token = self.consume_next_token();
        let declaration_name = if let Token::Ident(name) = next_token {
            name
        } else {
            panic!("Token is not a indent token");
        };
        let mut declaration = Declaration::new(declaration_name);
        self.consume_while_next_token_is(Token::Whitespace);

        match self.peek_next_token() {
            Token::Colon => {
                self.consume_next_token();
            }
            _ => {
                emit_error!("Expected Colon in declaration");
                return None;
            }
        }

        self.consume_while_next_token_is(Token::Whitespace);

        loop {
            let token = self.peek_next_token();
            if let Token::EOF = token {
                break;
            }
            declaration.append_value(self.consume_a_component_value());
        }

        let last_two_tokens = declaration.last_values(2);

        if last_two_tokens.len() == 2 {
            if let ComponentValue::PerservedToken(Token::Delim('!')) = last_two_tokens[0] {
                if let ComponentValue::PerservedToken(Token::Ident(data)) = last_two_tokens[1] {
                    if data.eq_ignore_ascii_case("important") {
                        declaration.pop_last(2);
                        declaration.important();
                    }
                }
            }
        }

        while let Some(Token::Whitespace) = declaration.last_token() {
            declaration.pop_last(1);
        }

        return Some(declaration);
    }

    fn consume_while_next_token_is(&mut self, token: Token) {
        while self.peek_next_token() == token {
            self.consume_next_token();
        }
    }
}

impl Parser {
    pub fn parse_a_stylesheet(&mut self) -> ListOfRules {
        self.top_level = true;
        let rules = self.consume_a_list_of_rules();
        return rules;
    }

    pub fn parse_a_css_stylesheet(&mut self) -> StyleSheet {
        let mut stylesheet = StyleSheet::new();
        let rules = self.parse_a_stylesheet();
        let mut parser = Parser::new(DataStream::new(Vec::new()));
        for rule in rules {
            if let Rule::QualifiedRule(rule) = rule {
                let selectors = parse_selectors(&rule.prelude);
                if selectors.len() == 0 {
                    // invalid rule
                    continue
                }
                let content = if let Some(block) = rule.block {
                    // transform component values to tokens
                    let tokens = block.value
                        .into_iter()
                        .filter_map(|com| {
                            match com {
                                ComponentValue::PerservedToken(t) => Some(t),
                                _ => None
                            }
                        })
                        .collect();
                    parser.recreate(DataStream::new(tokens));

                    let declarations = parser.parse_a_list_of_declarations();
                    // take only declaration
                    declarations
                        .into_iter()
                        .filter_map(|declaration| {
                            match declaration {
                                DeclarationOrAtRule::Declaration(d) => Some(d),
                                _ => None
                            }
                        })
                        .collect()
                } else {
                    Vec::new()
                };
                let style_rule = StyleRule::new(selectors, content);
                stylesheet.append_rule(CSSRule::Style(style_rule));
            }
            else {
                continue;
            }
        }
        stylesheet
    }

    pub fn parse_a_list_of_rules(&mut self) -> ListOfRules {
        self.top_level = false;
        let rules = self.consume_a_list_of_rules();
        return rules;
    }

    pub fn parse_a_rule(&mut self) -> Result<Rule, SyntaxError> {
        self.consume_while_next_token_is(Token::Whitespace);

        #[allow(unused_assignments)]
        let mut return_rule = None;

        if let Token::EOF = self.peek_next_token() {
            return Err(SyntaxError);
        } else if let Token::AtKeyword(_) = self.peek_next_token() {
            return_rule = Some(Rule::AtRule(self.consume_an_at_rule()));
        } else {
            if let Some(rule) = self.consume_a_qualified_rule() {
                return_rule = Some(Rule::QualifiedRule(rule));
            } else {
                return Err(SyntaxError);
            }
        }

        self.consume_while_next_token_is(Token::Whitespace);

        if let Token::EOF = self.peek_next_token() {
            return Ok(return_rule.unwrap());
        }
        return Err(SyntaxError);
    }

    pub fn parse_a_declaration(&mut self) -> Result<Declaration, SyntaxError> {
        self.consume_while_next_token_is(Token::Whitespace);
        if let Token::Ident(_) = self.peek_next_token() {
            if let Some(declaration) = self.consume_a_declaration() {
                return Ok(declaration);
            } else {
                return Err(SyntaxError);
            }
        }
        return Err(SyntaxError);
    }

    pub fn parse_a_list_of_declarations(&mut self) -> Vec<DeclarationOrAtRule> {
        return self.consume_a_list_of_declarations();
    }

    pub fn parse_a_component_value(&mut self) -> Result<ComponentValue, SyntaxError> {
        self.consume_while_next_token_is(Token::Whitespace);
        if let Token::EOF = self.peek_next_token() {
            return Err(SyntaxError);
        }
        let value = self.consume_a_component_value();
        self.consume_while_next_token_is(Token::Whitespace);
        if let Token::EOF = self.peek_next_token() {
            return Ok(value);
        }
        return Err(SyntaxError);
    }

    pub fn parse_a_list_of_component_values(&mut self) -> Vec<ComponentValue> {
        let mut values = Vec::new();
        loop {
            let value = self.consume_a_component_value();
            if let ComponentValue::PerservedToken(Token::EOF) = value {
                break;
            }
            values.push(value);
        }
        return values;
    }

    pub fn parse_a_comma_separated_list_of_component_values(&mut self) -> Vec<Vec<ComponentValue>> {
        let mut return_values = Vec::new();
        let mut values = Vec::new();
        loop {
            let value = self.consume_a_component_value();
            if let ComponentValue::PerservedToken(Token::EOF) = value {
                return_values.push(values.clone());
                break;
            }
            if let ComponentValue::PerservedToken(Token::Comma) = value {
                return_values.push(values.clone());
                values.clear();
                continue;
            }
            values.push(value);
        }
        return return_values;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::token::HashType;
    use crate::tokenizer::Tokenizer;

    #[test]
    fn parse_a_stylesheet() {
        let css = "div { color: black; }";
        let tokenizer = Tokenizer::new(css.to_string());
        let tokens = tokenizer.run();
        let mut parser = Parser::new(tokens);
        let rules = parser.parse_a_stylesheet();

        assert_eq!(rules.len(), 1);
        assert_eq!(
            rules[0],
            Rule::QualifiedRule(QualifiedRule {
                prelude: vec![
                    ComponentValue::PerservedToken(Token::Ident("div".to_string())),
                    ComponentValue::PerservedToken(Token::Whitespace)
                ],
                block: Some(SimpleBlock {
                    token: Token::BraceOpen,
                    value: vec![
                        ComponentValue::PerservedToken(Token::Whitespace),
                        ComponentValue::PerservedToken(Token::Ident("color".to_string())),
                        ComponentValue::PerservedToken(Token::Colon),
                        ComponentValue::PerservedToken(Token::Whitespace),
                        ComponentValue::PerservedToken(Token::Ident("black".to_string())),
                        ComponentValue::PerservedToken(Token::Semicolon),
                        ComponentValue::PerservedToken(Token::Whitespace),
                    ]
                })
            })
        );
    }

    #[test]
    fn parse_a_class() {
        let css = ".className { color: black; }";
        let tokenizer = Tokenizer::new(css.to_string());
        let tokens = tokenizer.run();
        let mut parser = Parser::new(tokens);
        let rules = parser.parse_a_stylesheet();

        assert_eq!(rules.len(), 1);
        assert_eq!(
            rules[0],
            Rule::QualifiedRule(QualifiedRule {
                prelude: vec![
                    ComponentValue::PerservedToken(Token::Delim('.')),
                    ComponentValue::PerservedToken(Token::Ident("className".to_string())),
                    ComponentValue::PerservedToken(Token::Whitespace)
                ],
                block: Some(SimpleBlock {
                    token: Token::BraceOpen,
                    value: vec![
                        ComponentValue::PerservedToken(Token::Whitespace),
                        ComponentValue::PerservedToken(Token::Ident("color".to_string())),
                        ComponentValue::PerservedToken(Token::Colon),
                        ComponentValue::PerservedToken(Token::Whitespace),
                        ComponentValue::PerservedToken(Token::Ident("black".to_string())),
                        ComponentValue::PerservedToken(Token::Semicolon),
                        ComponentValue::PerservedToken(Token::Whitespace),
                    ]
                })
            })
        );
    }

    #[test]
    fn parse_an_id() {
        let css = "#elementId { color: black; }";
        let tokenizer = Tokenizer::new(css.to_string());
        let tokens = tokenizer.run();
        let mut parser = Parser::new(tokens);
        let rules = parser.parse_a_stylesheet();

        assert_eq!(rules.len(), 1);
        assert_eq!(
            rules[0],
            Rule::QualifiedRule(QualifiedRule {
                prelude: vec![
                    ComponentValue::PerservedToken(Token::Hash(
                        "elementId".to_string(),
                        HashType::Id
                    )),
                    ComponentValue::PerservedToken(Token::Whitespace)
                ],
                block: Some(SimpleBlock {
                    token: Token::BraceOpen,
                    value: vec![
                        ComponentValue::PerservedToken(Token::Whitespace),
                        ComponentValue::PerservedToken(Token::Ident("color".to_string())),
                        ComponentValue::PerservedToken(Token::Colon),
                        ComponentValue::PerservedToken(Token::Whitespace),
                        ComponentValue::PerservedToken(Token::Ident("black".to_string())),
                        ComponentValue::PerservedToken(Token::Semicolon),
                        ComponentValue::PerservedToken(Token::Whitespace),
                    ]
                })
            })
        );
    }

    #[test]
    fn parse_css_stylesheet() {
        let css = "#elementId { color: black !important; }";
        let tokenizer = Tokenizer::new(css.to_string());
        let tokens = tokenizer.run();
        let mut parser = Parser::new(tokens);
        let stylesheet = parser.parse_a_css_stylesheet();
        println!("{:#?}", stylesheet);
    }
}
