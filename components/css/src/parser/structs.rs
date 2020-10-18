use crate::tokenizer::token::Token;

#[derive(Debug, PartialEq)]
pub enum Rule {
    QualifiedRule(QualifiedRule),
    AtRule(AtRule),
}
pub type ListOfRules = Vec<Rule>;

#[derive(Debug)]
pub enum DeclarationOrAtRule {
    Declaration(Declaration),
    AtRule(AtRule),
}

/// A simple block
/// https://www.w3.org/TR/css-syntax-3/#simple-block
#[derive(Clone, Debug, PartialEq)]
pub struct SimpleBlock {
    /// Associated token (either a <[-token>, <(-token>, or <{-token>)
    pub token: Token,
    /// Block value
    pub value: Vec<ComponentValue>,
}

/// Function
/// https://www.w3.org/TR/css-syntax-3/#function
#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub value: Vec<ComponentValue>,
}

/// QualifiedRule
/// https://www.w3.org/TR/css-syntax-3/#qualified-rule
#[derive(Debug, PartialEq)]
pub struct QualifiedRule {
    pub prelude: Vec<ComponentValue>,
    pub block: Option<SimpleBlock>,
}

/// AtRule
/// https://www.w3.org/TR/css-syntax-3/#at-rule
#[derive(Debug, PartialEq)]
pub struct AtRule {
    pub name: String,
    pub prelude: Vec<ComponentValue>,
    pub block: Option<SimpleBlock>,
}

/// Declaration
/// https://www.w3.org/TR/css-syntax-3/#declaration
#[derive(Debug, PartialEq)]
pub struct Declaration {
    pub name: String,
    pub value: Vec<ComponentValue>,
    pub important: bool,
}

/// ComponentValue
/// https://www.w3.org/TR/css-syntax-3/#component-value
#[derive(Clone, Debug, PartialEq)]
pub enum ComponentValue {
    PerservedToken(Token),
    Function(Function),
    SimpleBlock(SimpleBlock),
}

impl QualifiedRule {
    pub fn new() -> Self {
        Self {
            prelude: Vec::new(),
            block: None,
        }
    }

    pub fn set_block(&mut self, block: SimpleBlock) {
        self.block = Some(block);
    }

    pub fn append_prelude(&mut self, value: ComponentValue) {
        self.prelude.push(value);
    }
}

impl AtRule {
    pub fn new(name: String) -> Self {
        Self {
            name,
            prelude: Vec::new(),
            block: None,
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
            value: Vec::new(),
        }
    }

    pub fn append_value(&mut self, value: ComponentValue) {
        self.value.push(value);
    }
}

impl Declaration {
    pub fn new(name: String) -> Self {
        Self {
            name,
            value: Vec::new(),
            important: false,
        }
    }

    pub fn append_value(&mut self, value: ComponentValue) {
        self.value.push(value);
    }

    pub fn last_values(&self, len: usize) -> Vec<&ComponentValue> {
        self.value.iter().rev().take(len).rev().collect()
    }

    pub fn last_token(&self) -> Option<&Token> {
        for value in self.value.iter().rev() {
            if let ComponentValue::PerservedToken(token) = value {
                return Some(token);
            }
        }
        return None;
    }

    pub fn pop_last(&mut self, len: usize) {
        for _ in 0..len {
            self.value.pop();
        }
    }

    pub fn remove(&mut self, index: usize) {
        self.value.remove(index);
    }

    pub fn important(&mut self) {
        self.important = true;
    }

    pub fn tokens(&self) -> Vec<Token> {
        self.value
            .clone()
            .into_iter()
            .filter_map(|com| match com {
                ComponentValue::PerservedToken(t) => Some(t),
                _ => None,
            })
            .collect()
    }
}

impl Function {
    pub fn new(name: String) -> Self {
        Self {
            name,
            value: Vec::new(),
        }
    }

    pub fn append_value(&mut self, value: ComponentValue) {
        self.value.push(value);
    }
}
