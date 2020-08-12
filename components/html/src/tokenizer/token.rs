#[derive(Clone, Debug, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: String
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    DOCTYPE {
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

    pub fn new_doctype() -> Self {
        Token::DOCTYPE {
            name: None,
            public_identifier: None,
            system_identifier: None,
            force_quirks: false
        }
    }

    pub fn set_force_quirks(&mut self, value: bool) {
        if let Token::DOCTYPE { ref mut force_quirks, .. } = self {
            *force_quirks = value;
        }
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
