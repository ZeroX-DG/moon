pub enum Token {
    Ident(String),
    Function(String),
    AtKeyword(String),
    Hash(String, HashType),
    Str(String),
    BadStr,
    Url(String),
    BadUrl,
    Delim(char),
    Number {
        value: i32,
        type_: NumberType
    },
    Percentage(i32),
    Dimension {
        value: i32,
        type_: NumberType,
        unit: String
    },
    Whitespace,
    CDO,
    CDC,
    Colon,
    Semicolon,
    Comma,
    BracketOpen,
    BracketClose,
    ParentheseOpen,
    ParentheseClose,
    BraceOpen,
    BraceClose,
    EOF
}

pub enum HashType {
    Id,
    Unrestricted
}

pub enum NumberType {
    Integer,
    Number
}

impl Token {
    pub fn append_to_string_token(&mut self, ch: char) {
        if let Token::Str(ref mut data) = self {
            data.push(ch);
        }
    }

    pub fn new_hash() -> Self {
        Token::Hash(String::new(), HashType::Unrestricted)
    }

    pub fn set_hash_type(&mut self, new_type: HashType) {
        if let Token::Hash(_, ref mut type_) = self {
            *type_ = new_type;
        }
    }

    pub fn set_hash_value(&mut self, new_value: String) {
        if let Token::Hash(ref mut value, _) = self {
            *value = new_value;
        }
    }

    pub fn append_to_url_token(&mut self, ch: char) {
        if let Token::Url(ref mut data) = self {
            data.push(ch);
        }
    }
}
