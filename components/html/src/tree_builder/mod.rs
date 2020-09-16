mod insert_mode;
mod list_of_active_formatting_elements;
mod stack_of_open_elements;
mod open_element_types;

use super::element_factory::create_element;
use super::elements::HTMLScriptElement;
use super::tokenizer::state::State;
use super::tokenizer::token::Attribute;
use super::tokenizer::token::Token;
use super::tokenizer::Tokenizer;
use dom::comment::Comment;
use dom::document::{Document, DocumentType, QuirksMode};
use dom::dom_ref::NodeRef;
use dom::element::Element;
use dom::node::Node;
use dom::text::Text;
use insert_mode::InsertMode;
use list_of_active_formatting_elements::Entry;
use list_of_active_formatting_elements::ListOfActiveFormattingElements;
use stack_of_open_elements::StackOfOpenElements;
use open_element_types::is_special_element;
use std::env;

fn is_trace() -> bool {
    match env::var("TRACE_TREE_BUILDER") {
        Ok(s) => s == "true",
        _ => false,
    }
}

macro_rules! trace {
    ($err:expr) => {
        println!("[ParseError][TreeBuilding]: {}", $err);
    };
}

macro_rules! emit_error {
    ($err:expr) => {
        if is_trace() {
            trace!($err)
        }
    };
}

macro_rules! match_any {
    ($target:ident, $($cmp:expr), *) => {
        $($target == $cmp)||*
    };
    ($target:expr, $($cmp:expr), *) => {
        $($target == $cmp)||*
    };
}

/// Cast a node_ref to an Element. Only use when it is safe
macro_rules! get_element {
    ($target:ident) => {
        $target.borrow().as_element().expect("Node is not an element")
    };
    ($target:expr) => {
        $target.borrow().as_element().expect("Node is not an element")
    };
}

/// The DOM tree builder
pub struct TreeBuilder {
    /// The tokenizer controlled by TreeBuilder
    tokenizer: Tokenizer,

    /// Stack of open elements as mentioned in specs
    open_elements: StackOfOpenElements,

    /// Indicate if the tree builder should stop parsing
    should_stop: bool,

    /// Current insertion mode
    insert_mode: InsertMode,

    /// The insert mode that the builder will return
    original_insert_mode: Option<InsertMode>,

    /// The result document
    document: NodeRef,

    /// Enable or disable foster parenting
    foster_parenting: bool,

    /// Element pointer to head element
    head_pointer: Option<NodeRef>,

    /// Element pointer to last open form element
    form_pointer: Option<NodeRef>,

    /// Is scripting enable?
    scripting: bool,

    /// List of active formatting elements
    active_formatting_elements: ListOfActiveFormattingElements,

    /// `frameset_ok` flag
    frameset_ok: bool,

    /// Stack of template insert mode to parse nested template
    stack_of_template_insert_mode: Vec<InsertMode>,
}

/// The adjusted location to insert a node as mentioned the specs
pub struct AdjustedInsertionLocation {
    pub parent: NodeRef,
    pub insert_before_sibling: Option<NodeRef>,
}

/// The parsing algorithm to be used for parsing text-only element
enum TextOnlyElementParsingAlgo {
    GenericRawText,
    GenericRCDataElement,
}

/// Check if the character is a whitespace character according to specs
fn is_whitespace(c: char) -> bool {
    match c {
        '\t' | '\n' | '\x0C' | ' ' => true,
        _ => false,
    }
}

impl TreeBuilder {
    pub fn new(tokenizer: Tokenizer) -> Self {
        Self {
            tokenizer,
            open_elements: StackOfOpenElements::new(),
            insert_mode: InsertMode::Initial,
            document: NodeRef::new(Document::new()),
            foster_parenting: false,
            head_pointer: None,
            form_pointer: None,
            original_insert_mode: None,
            scripting: false,
            active_formatting_elements: ListOfActiveFormattingElements::new(),
            frameset_ok: true,
            stack_of_template_insert_mode: Vec::new(),
            should_stop: false,
        }
    }

    /// Start the main loop for parsing DOM tree
    pub fn run(&mut self) {
        loop {
            let token = self.tokenizer.next_token();
            if let Token::EOF = token {
                break;
            }
            self.process(token);
            if self.should_stop {
                break;
            }
        }
    }

    /// (Re)process a token in the current insert mode
    pub fn process(&mut self, token: Token) {
        match self.insert_mode {
            InsertMode::Initial => self.handle_initial(token),
            InsertMode::BeforeHtml => self.handle_before_html(token),
            InsertMode::BeforeHead => self.handle_before_head(token),
            InsertMode::InHead => self.handle_in_head(token),
            InsertMode::InHeadNoScript => self.handle_in_head_no_script(token),
            InsertMode::AfterHead => self.handle_after_head(token),
            InsertMode::InBody => self.handle_in_body(token),
            _ => unimplemented!(),
        }
    }

    /// Get the current parsing document
    pub fn get_document(&self) -> NodeRef {
        self.document.clone()
    }

    fn which_quirks_mode(&self, token: Token) -> QuirksMode {
        if let Token::DOCTYPE {
            name, force_quirks, ..
        } = token
        {
            // TODO: Implement full stpecs detection
            if force_quirks || name.unwrap_or_default() != "html" {
                return QuirksMode::Quirks;
            }
        }
        QuirksMode::NoQuirks
    }

    fn switch_to(&mut self, mode: InsertMode) {
        println!("Switch to: {:#?}", mode);
        self.insert_mode = mode;
    }

    fn stop_parsing(&mut self) {
        self.should_stop = true;
    }

    fn create_element(&self, tag_token: Token) -> NodeRef {
        let (tag_name, attributes) = if let Token::Tag {
            tag_name,
            attributes,
            ..
        } = tag_token
        {
            (tag_name, attributes)
        } else {
            ("".to_string(), Vec::new())
        };
        let element_ref = create_element(self.document.clone().downgrade(), &tag_name);
        if let Some(element) = element_ref
            .borrow_mut()
            .as_any_mut()
            .downcast_mut::<Element>()
        {
            for attribute in attributes {
                element.set_attribute(&attribute.name, &attribute.value);
            }
        }
        element_ref
    }

    fn create_element_from_tag_name(&self, tag_name: &str) -> NodeRef {
        self.create_element(Token::Tag {
            tag_name: tag_name.to_owned(),
            self_closing: false,
            attributes: Vec::new(),
            is_end_tag: false,
            self_closing_acknowledged: false,
        })
    }

    fn get_appropriate_place_for_inserting_a_node(&self) -> AdjustedInsertionLocation {
        let target = self.open_elements.current_node().unwrap();

        // TODO: implement full specs
        return AdjustedInsertionLocation {
            parent: target.clone(),
            insert_before_sibling: target.borrow().as_node().last_child(),
        };
    }

    fn insert_html_element(&mut self, token: Token) -> NodeRef {
        let insert_position = self.get_appropriate_place_for_inserting_a_node();
        let element = self.create_element(token);
        let return_ref = element.clone();

        // TODO: check if location is possible to insert node (Idk why so we just leave it for now)
        self.open_elements.push(element.clone());
        Node::insert_before(
            insert_position.parent,
            element,
            insert_position.insert_before_sibling,
        );
        return_ref
    }

    fn insert_character(&mut self, ch: char) {
        let insert_position = self.get_appropriate_place_for_inserting_a_node();
        if insert_position
            .parent
            .borrow()
            .as_any()
            .downcast_ref::<Document>()
            .is_some()
        {
            return;
        }
        if let Some(sibling) = insert_position.insert_before_sibling.clone() {
            if let Some(text) = sibling.borrow_mut().as_any_mut().downcast_mut::<Text>() {
                text.character_data.append_data(&ch.to_string());
                return;
            }
        }
        let text = NodeRef::new(Text::new(ch.to_string()));
        Node::insert_before(
            insert_position.parent,
            text,
            insert_position.insert_before_sibling,
        );
    }

    fn insert_comment(&mut self, data: String) {
        let insert_position = self.get_appropriate_place_for_inserting_a_node();
        let comment = NodeRef::new(Comment::new(data));
        Node::insert_before(
            insert_position.parent,
            comment,
            insert_position.insert_before_sibling,
        );
    }

    fn handle_text_only_element(&mut self, token: Token, algorithm: TextOnlyElementParsingAlgo) {
        self.insert_html_element(token);
        match algorithm {
            TextOnlyElementParsingAlgo::GenericRawText => {
                self.tokenizer.switch_to(State::RAWTEXT);
            }
            TextOnlyElementParsingAlgo::GenericRCDataElement => {
                self.tokenizer.switch_to(State::RCDATA);
            }
        }
        self.original_insert_mode = Some(self.insert_mode.clone());
        self.switch_to(InsertMode::Text);
    }

    fn generate_all_implied_end_tags_thoroughly(&mut self) {
        while let Some(node) = self.open_elements.current_node() {
            let node = node.borrow();
            let element = node.as_element().unwrap();
            let tag_name = element.tag_name();
            if match_any!(
                tag_name, "caption", "colgroup", "dd", "dt", "li", "optgroup", "option", "p", "rb",
                "rt", "rtc", "tbody", "td", "tfoot", "th", "thead", "tr"
            ) {
                self.open_elements.pop();
            } else {
                break;
            }
        }
    }

    fn reset_insertion_mode_appropriately(&mut self) {
        unimplemented!(); // TODO: Implement this when supporting template tag
    }

    fn is_marker_or_open_element(&self, entry: &Entry) -> bool {
        if let Entry::Marker = entry {
            return true;
        }

        if let Entry::Element(element) = entry {
            if self.open_elements.contains_node(&element) {
                return true;
            }
        }

        false
    }

    fn unexpected(&self, token: &Token) {
        match token {
            Token::Tag { .. } => emit_error!("Unexpected tag"),
            Token::DOCTYPE { .. } => emit_error!("Unexpected DOCTYPE"),
            Token::Comment(_) => emit_error!("Unexpected comment"),
            Token::Character(_) => emit_error!("Unexpected character"),
            Token::EOF => emit_error!("Unexpected EOF"),
        }
    }

    fn close_p_element(&mut self) {
        self.generate_implied_end_tags("p");

        if get_element!(self.open_elements.current_node().unwrap()).tag_name() != "p" {
            emit_error!("Expected p element");
        }

        self.open_elements.pop_until("p");
    }

    fn generate_implied_end_tags(&mut self, exclude: &str) {
        while let Some(node) = self.open_elements.current_node() {
            let node = node.borrow();
            let element = node.as_element().unwrap();
            let tag_name = element.tag_name();
            if tag_name != exclude
                && match_any!(
                    tag_name, "dd", "dt", "li", "optgroup", "option", "p", "rb", "rt", "rtc", "rp"
                )
            {
                self.open_elements.pop();
            } else {
                break;
            }
        }
    }

    fn current_node(&self) -> NodeRef {
        self.open_elements.current_node().unwrap()
    }

    fn reconstruct_active_formatting_elements(&mut self) {
        if self.active_formatting_elements.len() == 0 {
            return;
        }

        let last_active_element = self.active_formatting_elements.last().unwrap();

        if self.is_marker_or_open_element(last_active_element) {
            return;
        }

        let mut last_index = self.active_formatting_elements.len() - 1;

        loop {
            if last_index == 0 {
                break;
            }

            last_index -= 1;
            let entry = &self.active_formatting_elements[last_index];

            if self.is_marker_or_open_element(entry) {
                break;
            }
        }

        // advance step
        last_index += 1;

        loop {
            let element = match &self.active_formatting_elements[last_index] {
                Entry::Element(element) => element.clone(),
                Entry::Marker => panic!("Unexpected marker while building DOM tree!"),
            };

            let element = element.borrow();
            let tag = element.as_element().unwrap();

            let new_element = self.insert_html_element(Token::Tag {
                is_end_tag: false,
                self_closing: false,
                self_closing_acknowledged: false,
                tag_name: tag.tag_name(),
                attributes: tag
                    .attributes()
                    .iter()
                    .map(|entry| Attribute {
                        name: entry.0.clone(),
                        value: entry.1.clone(),
                    })
                    .collect(),
            });

            self.active_formatting_elements[last_index] = Entry::Element(new_element);

            if last_index == self.active_formatting_elements.len() - 1 {
                break;
            }
            last_index += 1;
        }
    }
}

impl TreeBuilder {
    fn handle_initial(&mut self, token: Token) {
        if let Token::Character(c) = token {
            if is_whitespace(c) {
                return;
            }
        }

        if let Token::Comment(data) = token {
            let comment = NodeRef::new(Comment::new(data));
            Node::append_child(self.document.clone(), comment);
            return;
        }

        if let Token::DOCTYPE {
            name,
            public_identifier,
            system_identifier,
            ..
        } = token.clone()
        {
            let name = name.unwrap_or_default();

            let error = match (
                name.as_str(),
                public_identifier.clone(),
                system_identifier.clone(),
            ) {
                ("html", None, None) => false,
                ("html", None, Some(c)) if c == "about:legacy-compat" => false,
                _ => true,
            };

            if error {
                self.unexpected(&token)
            }

            let doctype = DocumentType::new(name, public_identifier, system_identifier);

            if let Some(doc) = self
                .document
                .borrow_mut()
                .as_any_mut()
                .downcast_mut::<Document>()
            {
                doc.set_doctype(doctype);
                doc.set_mode(self.which_quirks_mode(token));
            }

            self.switch_to(InsertMode::BeforeHtml);
            return;
        }

        self.unexpected(&token);
        self.switch_to(InsertMode::BeforeHtml);
        self.process(token)
    }

    fn handle_before_html(&mut self, token: Token) {
        fn anything_else(this: &mut TreeBuilder, token: Token) {
            let element = this.create_element_from_tag_name("html");
            Node::append_child(this.document.clone(), element.clone());
            this.open_elements.push(element.clone());
            // TODO: Implement additional steps in specs
            this.switch_to(InsertMode::BeforeHead);
            this.process(token.clone())
        }

        if let Token::DOCTYPE { .. } = token {
            self.unexpected(&token);
            return;
        }

        if let Token::Comment(data) = token {
            let comment = NodeRef::new(Comment::new(data));
            Node::append_child(self.document.clone(), comment);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "html" {
            let element = self.create_element(token);
            Node::append_child(self.document.clone(), element.clone());
            self.open_elements.push(element.clone());
            // TODO: implement additional steps in specs
            self.switch_to(InsertMode::BeforeHead);
            return;
        }

        if token.is_end_tag() && match_any!(token.tag_name(), "head", "body", "html", "br") {
            anything_else(self, token);
            return;
        }

        if token.is_end_tag() {
            self.unexpected(&token);
            anything_else(self, token);
            return;
        }

        anything_else(self, token);
    }

    fn handle_before_head(&mut self, token: Token) {
        fn anything_else(this: &mut TreeBuilder, token: Token) {
            let head_element = this.insert_html_element(Token::Tag {
                tag_name: "head".to_owned(),
                attributes: Vec::new(),
                is_end_tag: false,
                self_closing: false,
                self_closing_acknowledged: false,
            });
            this.head_pointer = Some(head_element.clone());
            this.switch_to(InsertMode::InHead);
            this.process(token)
        }

        if let Token::Character(c) = token {
            if is_whitespace(c) {
                return;
            }
        }

        if let Token::Comment(data) = token {
            self.insert_comment(data);
            return;
        }

        if let Token::DOCTYPE { .. } = token {
            self.unexpected(&token);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "html" {
            return self.handle_in_body(token);
        }

        if token.is_start_tag() && token.tag_name() == "head" {
            let head_element = self.insert_html_element(token);
            self.head_pointer = Some(head_element);
            self.switch_to(InsertMode::InHead);
            return;
        }

        if token.is_end_tag() && match_any!(token.tag_name(), "head", "body", "html", "br") {
            return anything_else(self, token);
        }

        if token.is_end_tag() {
            self.unexpected(&token);
            return;
        }

        anything_else(self, token);
    }

    fn handle_in_head(&mut self, mut token: Token) {
        if let Token::Character(c) = token {
            if is_whitespace(c) {
                self.insert_character(c);
                return;
            }
        }

        if let Token::Comment(data) = token {
            self.insert_comment(data);
            return;
        }

        if let Token::DOCTYPE { .. } = token {
            self.unexpected(&token);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "html" {
            return self.handle_in_body(token);
        }

        if token.is_start_tag()
            && match_any!(token.tag_name(), "base", "basefont", "bgsound", "link")
        {
            self.insert_html_element(token.clone());
            self.open_elements.pop();
            token.acknowledge_self_closing_if_set();
            return;
        }

        if token.is_start_tag() && token.tag_name() == "meta" {
            self.insert_html_element(token.clone());
            self.open_elements.pop();
            token.acknowledge_self_closing_if_set();
            return;
        }

        if token.is_start_tag() && token.tag_name() == "title" {
            self.handle_text_only_element(token, TextOnlyElementParsingAlgo::GenericRCDataElement);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "noscript" && self.scripting {
            self.handle_text_only_element(token, TextOnlyElementParsingAlgo::GenericRawText);
            return;
        }

        if token.is_start_tag() && match_any!(token.tag_name(), "noframes", "style") {
            self.handle_text_only_element(token, TextOnlyElementParsingAlgo::GenericRawText);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "noscript" && !self.scripting {
            self.insert_html_element(token);
            self.switch_to(InsertMode::InHeadNoScript);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "script" {
            let insert_position = self.get_appropriate_place_for_inserting_a_node();
            let element = self.create_element(token);
            if let Some(script_element) = element
                .borrow_mut()
                .as_any_mut()
                .downcast_mut::<HTMLScriptElement>()
            {
                script_element.set_non_blocking(false);
                script_element.set_parser_document(self.get_document());
            }

            // TODO: implement steps 4 and 5

            Node::insert_before(
                insert_position.parent,
                element.clone(),
                insert_position.insert_before_sibling,
            );
            self.open_elements.push(element.clone());
            self.tokenizer.switch_to(State::ScriptData);
            self.original_insert_mode = Some(self.insert_mode.clone());
            self.switch_to(InsertMode::Text);
            return;
        }

        if token.is_end_tag() && token.tag_name() == "head" {
            self.open_elements.pop();
            self.switch_to(InsertMode::AfterHead);
            return;
        }

        if token.is_end_tag() && match_any!(token.tag_name(), "body", "html", "br") {
            self.open_elements.pop();
            self.switch_to(InsertMode::AfterHead);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "template" {
            self.insert_html_element(token);
            self.active_formatting_elements.add_marker();
            self.frameset_ok = false;
            self.switch_to(InsertMode::InTemplate);
            self.stack_of_template_insert_mode
                .push(InsertMode::InTemplate);
            return;
        }

        if token.is_end_tag() && token.tag_name() == "template" {
            if !self.open_elements.contains("template") {
                emit_error!("No template tag found");
                return;
            }

            self.generate_all_implied_end_tags_thoroughly();

            if let Some(node) = self.open_elements.current_node() {
                let node = node.borrow();
                let element = node.as_element().unwrap();
                if element.tag_name() != "template" {
                    emit_error!("Expected current node to be template");
                }
            }

            self.open_elements.pop_until("template");
            self.active_formatting_elements.clear_up_to_last_marker();
            self.stack_of_template_insert_mode.pop();
            self.reset_insertion_mode_appropriately();
            return;
        }

        if token.is_start_tag() && token.tag_name() == "head" {
            self.unexpected(&token);
            return;
        }

        if token.is_end_tag() {
            self.unexpected(&token);
            return;
        }

        self.open_elements.pop();
        self.switch_to(InsertMode::AfterHead);
        self.process(token)
    }

    fn handle_in_head_no_script(&mut self, token: Token) {
        fn anything_else(this: &mut TreeBuilder, token: Token) {
            this.unexpected(&token);
            this.open_elements.pop();
            this.switch_to(InsertMode::InHead);
            this.process(token)
        }

        if let Token::DOCTYPE { .. } = token {
            self.unexpected(&token);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "html" {
            return self.handle_in_body(token);
        }

        if token.is_end_tag() && token.tag_name() == "noscript" {
            self.open_elements.pop();
            self.switch_to(InsertMode::InHead);
            return;
        }

        if let Token::Character(c) = token {
            if is_whitespace(c) {
                return self.handle_in_head(token);
            }
        }

        if let Token::Comment(_) = token {
            return self.handle_in_head(token);
        }

        if token.is_start_tag()
            && match_any!(
                token.tag_name(),
                "basefont",
                "bgsound",
                "link",
                "meta",
                "noframes",
                "style"
            )
        {
            return self.handle_in_head(token);
        }

        if token.is_end_tag() && token.tag_name() == "br" {
            return anything_else(self, token);
        }

        if token.is_start_tag() && match_any!(token.tag_name(), "head", "noscript") {
            self.unexpected(&token);
            return;
        }

        if token.is_end_tag() {
            self.unexpected(&token);
            return;
        }

        anything_else(self, token)
    }

    fn handle_after_head(&mut self, token: Token) {
        fn anything_else(this: &mut TreeBuilder, token: Token) {
            this.insert_html_element(Token::Tag {
                is_end_tag: false,
                tag_name: "body".to_owned(),
                self_closing: false,
                self_closing_acknowledged: false,
                attributes: Vec::new(),
            });
            this.switch_to(InsertMode::InBody);
            this.process(token)
        }

        if let Token::Character(c) = token {
            if is_whitespace(c) {
                self.insert_character(c);
                return;
            }
        }

        if let Token::Comment(data) = token {
            self.insert_comment(data);
            return;
        }

        if let Token::DOCTYPE { .. } = token {
            self.unexpected(&token);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "html" {
            return self.handle_in_body(token);
        }

        if token.is_start_tag() && token.tag_name() == "body" {
            self.insert_html_element(token);
            self.frameset_ok = false;
            self.switch_to(InsertMode::InBody);
            return;
        }

        if token.is_start_tag() && token.tag_name() == "frameset" {
            self.insert_html_element(token);
            self.switch_to(InsertMode::InFrameset);
            return;
        }

        if token.is_start_tag()
            && match_any!(
                token.tag_name(),
                "base",
                "basefont",
                "bgsound",
                "link",
                "meta",
                "noframes",
                "script",
                "style",
                "template",
                "title"
            )
        {
            self.unexpected(&token);
            let head = self.head_pointer.clone().unwrap();
            self.open_elements.push(head.clone());
            self.handle_in_head(token);
            self.open_elements
                .remove_first_matching(|node| node.clone() == head);
            return;
        }

        if token.is_end_tag() && token.tag_name() == "template" {
            return self.handle_in_head(token);
        }

        if token.is_end_tag() && match_any!(token.tag_name(), "body", "html", "br") {
            return anything_else(self, token);
        }

        if token.is_start_tag() && token.tag_name() == "head" {
            self.unexpected(&token);
            return;
        }

        if token.is_end_tag() {
            self.unexpected(&token);
            return;
        }

        anything_else(self, token)
    }

    fn handle_in_body(&mut self, token: Token) {
        if let Token::Character(c) = token {
            if c == '\0' {
                emit_error!("Unexpected null character");
                return;
            }

            if is_whitespace(c) {
                self.reconstruct_active_formatting_elements();
                self.insert_character(c);
                return;
            }

            self.reconstruct_active_formatting_elements();
            self.insert_character(c);
            self.frameset_ok = false;
            return;
        }

        if let Token::Comment(data) = token {
            self.insert_comment(data);
            return;
        }

        if let Token::DOCTYPE { .. } = token {
            emit_error!("Unexpected DOCTYPE");
            return;
        }

        if token.is_start_tag() && token.tag_name() == "html" {
            emit_error!("Unexpected HTML tag");
            if self.open_elements.contains("template") {
                return;
            }

            let current_node = self.open_elements.current_node().unwrap();
            let mut current_node = current_node.borrow_mut();

            let current_element = current_node.as_element_mut().unwrap();

            for attribute in token.attributes() {
                if current_element.has_attribute(&attribute.name) {
                    continue;
                }
                current_element.set_attribute(&attribute.name, &attribute.value);
            }
            return;
        }

        if token.is_start_tag()
            && match_any!(
                token.tag_name(),
                "base",
                "basefont",
                "bgsound",
                "link",
                "meta",
                "noframes",
                "script",
                "style",
                "template",
                "title"
            )
        {
            return self.handle_in_head(token);
        }

        if token.is_end_tag() && token.tag_name() == "template" {
            return self.handle_in_head(token);
        }

        if token.is_start_tag() && token.tag_name() == "body" {
            self.unexpected(&token);
            if self.open_elements.len() == 1 {
                return;
            }

            if let Some(element) = self.open_elements.get(1).borrow().as_element() {
                if element.tag_name() != "body" {
                    return;
                }
            }

            if self.open_elements.contains("template") {
                return;
            }

            self.frameset_ok = false;
            let body = self.open_elements.get(1);
            let mut body = body.borrow_mut();
            let body = body.as_element_mut().unwrap();
            for attribute in token.attributes() {
                if body.has_attribute(&attribute.name) {
                    continue;
                }
                body.set_attribute(&attribute.name, &attribute.value);
            }
        }

        if token.is_start_tag() && token.tag_name() == "frameset" {
            self.unexpected(&token);
            if self.open_elements.len() == 1 {
                return;
            }

            if let Some(element) = self.open_elements.get(1).borrow().as_element() {
                if element.tag_name() != "body" {
                    return;
                }
            }

            if !self.frameset_ok {
                return;
            }

            // TODO: implement the rest of specs
        }

        if token.is_eof() {
            if self.stack_of_template_insert_mode.len() > 0 {
                return self.handle_in_template(token);
            }

            if self.open_elements.any(|node| {
                !(match_any!(
                    node.borrow().as_element().unwrap().tag_name(),
                    "dd",
                    "dt",
                    "li",
                    "optgroup",
                    "option",
                    "p",
                    "rb",
                    "rp",
                    "rt",
                    "rtc",
                    "tbody",
                    "td",
                    "tfoot",
                    "th",
                    "thead",
                    "tr",
                    "body",
                    "html"
                ))
            }) {
                self.unexpected(&token);
            }

            self.stop_parsing();
            return;
        }

        if token.is_end_tag() && token.tag_name() == "body" {
            if self.open_elements.has_element_name_in_scope("body") {
                self.unexpected(&token);
                return;
            }
            if self.open_elements.any(|node| {
                !(match_any!(
                    get_element!(node).tag_name(),
                    "dd",
                    "dt",
                    "li",
                    "optgroup",
                    "option",
                    "p",
                    "rb",
                    "rp",
                    "rt",
                    "rtc",
                    "tbody",
                    "td",
                    "tfoot",
                    "th",
                    "thead",
                    "tr",
                    "body",
                    "html"
                ))
            }) {
                self.unexpected(&token);
            }
            self.switch_to(InsertMode::AfterBody);
            return;
        }

        if token.is_end_tag() && token.tag_name() == "html" {
            if self.open_elements.has_element_name_in_scope("body") {
                self.unexpected(&token);
                return;
            }
            if self.open_elements.any(|node| {
                !(match_any!(
                    get_element!(node).tag_name(),
                    "dd",
                    "dt",
                    "li",
                    "optgroup",
                    "option",
                    "p",
                    "rb",
                    "rp",
                    "rt",
                    "rtc",
                    "tbody",
                    "td",
                    "tfoot",
                    "th",
                    "thead",
                    "tr",
                    "body",
                    "html"
                ))
            }) {
                self.unexpected(&token);
            }
            self.switch_to(InsertMode::AfterBody);
            return self.process(token);
        }

        if token.is_start_tag()
            && match_any!(
                token.tag_name(),
                "address",
                "article",
                "aside",
                "blockquote",
                "center",
                "details",
                "dialog",
                "dir",
                "div",
                "dl",
                "fieldset",
                "figcaption",
                "figure",
                "footer",
                "header",
                "hgroup",
                "main",
                "menu",
                "nav",
                "ol",
                "p",
                "section",
                "summary",
                "ul"
            )
        {
            if self.open_elements.has_element_name_in_button_scope("p") {
                self.close_p_element();
            }

            self.insert_html_element(token);
            return;
        }

        if token.is_start_tag() && match_any!(token.tag_name(), "h1", "h2", "h3", "h4", "h5", "h6")
        {
            if self.open_elements.has_element_name_in_button_scope("p") {
                self.close_p_element();
            }

            let current_tag_name = get_element!(self
                .open_elements
                .current_node()
                .unwrap())
                .tag_name();

            if match_any!(current_tag_name, "h1", "h2", "h3", "h4", "h5", "h6") {
                self.unexpected(&token);
                self.open_elements.pop();
            }

            self.insert_html_element(token);
            return;
        }

        if token.is_start_tag() && match_any!(token.tag_name(), "pre", "listing") {
            if self.open_elements.has_element_name_in_button_scope("p") {
                self.close_p_element();
            }

            self.insert_html_element(token);

            let next_token = self.tokenizer.next_token();

            self.frameset_ok = false;

            if let Token::Character(c) = next_token {
                if c == '\n' {
                    // ignore the token
                } else {
                    self.process(next_token);
                }
            }
            return
        }

        if token.is_start_tag() && token.tag_name() == "form" {
            let has_template_on_stack = self.open_elements.contains("template");
            if self.form_pointer.is_some() && !has_template_on_stack {
                self.unexpected(&token);
                return
            }

            if self.open_elements.has_element_name_in_button_scope("p") {
                self.close_p_element();
            }

            let form_element = self.insert_html_element(token);
            if !has_template_on_stack {
                self.form_pointer = Some(form_element);
            }
            return
        }

        if token.is_start_tag() && token.tag_name() == "li" {
            self.frameset_ok = false;
            for node in self.open_elements.0.iter().rev() {
                let element_tag_name = get_element!(node).tag_name();
                if element_tag_name == "li" {
                    self.generate_implied_end_tags("li");
                    if get_element!(self.current_node()).tag_name() != "li" {
                        emit_error!("Expected 'li' tag");
                    }
                    self.open_elements.pop_until("li");
                    break
                }
                
                if !match_any!(element_tag_name, "address", "div", "p") &&
                    is_special_element(&element_tag_name) {
                    break
                }
            }

            if self.open_elements.has_element_name_in_button_scope("p") {
                self.close_p_element();
            }
            self.insert_html_element(token);
            return
        }

        if token.is_start_tag() && match_any!(token.tag_name(), "dd", "dt") {
            self.frameset_ok = false;
            for node in self.open_elements.0.iter().rev() {
                let element_tag_name = get_element!(node).tag_name();
                if element_tag_name == "dd" {
                    self.generate_implied_end_tags("dd");
                    if get_element!(self.current_node()).tag_name() != "dd" {
                        emit_error!("Expected 'dd' tag");
                    }
                    self.open_elements.pop_until("dd");
                    break
                }

                if element_tag_name == "dt" {
                    self.generate_implied_end_tags("dt");
                    if get_element!(self.current_node()).tag_name() != "dt" {
                        emit_error!("Expected 'dt' tag");
                    }
                    self.open_elements.pop_until("dt");
                    break
                }
                
                if !match_any!(element_tag_name, "address", "div", "p") &&
                    is_special_element(&element_tag_name) {
                    break
                }
            } 

            if self.open_elements.has_element_name_in_button_scope("p") {
                self.close_p_element();
            }
            self.insert_html_element(token);
            return
        }

        if token.is_start_tag() && token.tag_name() == "plaintext" {
            if self.open_elements.has_element_name_in_button_scope("p") {
                self.close_p_element();
            }
            self.insert_html_element(token);
            self.tokenizer.switch_to(State::PLAINTEXT);
            return
        }

        if token.is_start_tag() && token.tag_name() == "button" {
            if self.open_elements.has_element_name_in_scope("button") {
                self.unexpected(&token);
                self.generate_implied_end_tags("");
                self.open_elements.pop_until("button");
            }
            self.reconstruct_active_formatting_elements();
            self.insert_html_element(token);
            self.frameset_ok = false;
            return
        }

        if token.is_end_tag() && match_any!(token.tag_name(), "address", "article", "aside", "blockquote", "button", "center", "details", "dialog", "dir", "div", "dl", "fieldset", "figcaption", "figure", "footer", "header", "hgroup", "listing", "main", "menu", "nav", "ol", "pre", "section", "summary", "ul") {
            if self.open_elements.has_element_name_in_scope(&token.tag_name()) {
                self.unexpected(&token);
                return
            }

            self.generate_implied_end_tags("");
            if get_element!(self.current_node()).tag_name() != *token.tag_name() {
                self.unexpected(&token);
                return
            }
            self.open_elements.pop_until(&token.tag_name());
            return
        }

        if token.is_end_tag() && token.tag_name() == "form" {
            if !self.open_elements.contains("template") {
                let node = self.form_pointer.clone();
                self.form_pointer = None;

                if node.is_none() {
                    self.unexpected(&token);
                    return
                }

                let node = node.unwrap();

                if self.open_elements.has_element_in_scope(&node) {
                    self.unexpected(&token);
                    return
                }

                self.generate_implied_end_tags("");

                if self.current_node() != node {
                    self.unexpected(&token);
                }

                self.open_elements.remove_first_matching(|fnode| *fnode == node);
            } else {
                if !self.open_elements.has_element_name_in_scope("form") {
                    self.unexpected(&token);
                    return
                }
                self.generate_implied_end_tags("");
                if get_element!(self.current_node()).tag_name() != "form" {
                    self.unexpected(&token);
                }
                self.open_elements.pop_until("form");
            }
            return
        }

        if token.is_end_tag() && token.tag_name() == "p" {
            if !self.open_elements.has_element_name_in_button_scope("p") {
                self.unexpected(&token);
                self.insert_html_element(Token::new_start_tag_with_name("p"));
            }
            self.close_p_element();
            return
        }
    }

    fn handle_in_template(&mut self, token: Token) {}
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tokenizer::Tokenizer;

    #[test]
    fn handle_initial_correctly() {
        let html = "<!-- this is a test -->".to_owned();
        let tokenizer = Tokenizer::new(html);
        let mut tree_builder = TreeBuilder::new(tokenizer);

        tree_builder.run();

        assert_eq!(
            tree_builder
                .get_document()
                .borrow()
                .as_node()
                .first_child()
                .unwrap()
                .borrow()
                .as_any()
                .downcast_ref::<Comment>()
                .unwrap()
                .get_data(),
            " this is a test "
        );
    }

    #[test]
    fn get_element_success() {
        let element = Element::new("div".to_owned());
        let node = NodeRef::new(element);
        assert!(get_element!(node).tag_name() == "div");
    }
}
