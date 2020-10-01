use std::env;
use super::tokenizer::Tokenizer;
use super::tokenizer::token::Token;
use cssom::stylesheet::StyleSheet;
use cssom::css_rule::CSSRule;

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

/// CSS Parser
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

// TODO: Support at-rule too
type Rule = QualifiedRule;
type ListOfRules = Vec<Rule>;

/// A simple block
/// https://www.w3.org/TR/css-syntax-3/#simple-block
pub struct SimpleBlock {
    /// Associated token (either a <[-token>, <(-token>, or <{-token>)
    token: Token,
    /// Block value
    value: Vec<ComponentValue>
}

/// Function
/// https://www.w3.org/TR/css-syntax-3/#function
pub struct Function {
    name: String,
    value: Vec<ComponentValue>
}

/// QualifiedRule
/// https://www.w3.org/TR/css-syntax-3/#qualified-rule
pub struct QualifiedRule {
    prelude: Vec<ComponentValue>,
    block: Option<SimpleBlock>
}

/// ComponentValue
/// https://www.w3.org/TR/css-syntax-3/#component-value
pub enum ComponentValue {
    PerservedToken(Token),
    Function(Function),
    SimpleBlock(SimpleBlock)
}

impl QualifiedRule {
    pub fn new() -> Self {
        Self {
            prelude: Vec::new(),
            block: None
        }
    }

    pub fn set_block(&mut self, block: SimpleBlock) {
        self.block = Some(block);
    }

    pub fn append_prelude(&mut self, value: ComponentValue) {
        self.prelude.push(value);
    }
}

impl SimpleBlock {
    pub fn new(token: Token) -> Self {
        Self {
            token,
            value: Vec::new()
        }
    }

    pub fn append_value(&mut self, value: ComponentValue) {
        self.value.push(value);
    }
}

impl Function {
    pub fn new(name: String) -> Self {
        Self {
            name,
            value: Vec::new()
        }
    }

    pub fn append_value(&mut self, value: ComponentValue) {
        self.value.push(value);
    }
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

    fn ending_token(&self) -> Token {
        match self.current_token {
            Some(Token::BracketOpen) => Token::BracketClose,
            Some(Token::BraceOpen) => Token::BraceClose,
            Some(Token::ParentheseOpen) => Token::ParentheseClose,
            _ => panic!("Can't get ending token")
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
        self.consume_next_token();

        match self.current_token.clone().unwrap() {
            Token::BraceOpen | Token::BracketOpen | Token::ParentheseOpen => {
                return ComponentValue::SimpleBlock(self.consume_a_simple_block());
            }
            Token::Function(_) => {
                return ComponentValue::Function(self.consume_a_function());
            }
            t => {
                return ComponentValue::PerservedToken(t)
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
    pub fn parse_a_stylesheet(&mut self) -> StyleSheet {
        self.top_level = true;
        let rules = self.consume_a_list_of_rules();
        unimplemented!()
    }

    pub fn parse_a_list_of_rules(&mut self) -> ListOfRules {
        self.top_level = false;
        let rules = self.consume_a_list_of_rules();
        return rules;
    }
}
