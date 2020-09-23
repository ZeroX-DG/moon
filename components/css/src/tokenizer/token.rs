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
    Number(usize),
    Percentage(usize),
    Dimension(usize, String),
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
