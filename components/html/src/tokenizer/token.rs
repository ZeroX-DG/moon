#[derive(Clone, Debug, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: String
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Doctype {
        name: Option<String>,
        public_identifier: Option<String>,
        system_identifier: Option<String>,
        force_quirks: bool
    },
    Tag {
        tag_name: String,
        self_closing: bool,
        attributes: Vec<Attribute>,
        is_end_tag: bool
    },
    Comment(String),
    Character(char),
    EOF
}

impl Token {
    pub fn new_start_tag() -> Self {
        Token::Tag {
            tag_name: String::new(),
            is_end_tag: false,
            self_closing: false,
            attributes: Vec::new()
        }
    }

    pub fn new_end_tag() -> Self {
        Token::Tag {
            tag_name: String::new(),
            is_end_tag: true,
            self_closing: false,
            attributes: Vec::new()
        }
    }

    pub fn new_comment(data: &str) -> Self {
        Token::Comment(data.to_owned())
    }
}

impl Attribute {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            value: String::new()
        }
    }
}
