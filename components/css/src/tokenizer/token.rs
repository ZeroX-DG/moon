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
        value: usize,
        type_: NumberType
    },
    Percentage(usize),
    Dimension {
        value: usize,
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
}
