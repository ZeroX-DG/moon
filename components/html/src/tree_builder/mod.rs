mod insert_mode;
mod stack_of_open_elements;
mod list_of_active_formatting_elements;

use std::env;
use insert_mode::InsertMode;
use stack_of_open_elements::StackOfOpenElements;
use list_of_active_formatting_elements::ListOfActiveFormattingElements;
use super::tokenizer::token::Token;
use super::tokenizer::state::State;
use super::tokenizer::Tokenizer;
use dom::dom_ref::NodeRef;
use dom::document::{Document, QuirksMode, DocumentType};
use dom::comment::Comment;
use dom::element::Element;
use dom::node::Node;
use dom::text::Text;
use super::element_factory::create_element;
use super::elements::HTMLScriptElement;

fn is_trace() -> bool {
    match env::var("TRACE_TREE_BUILDER") {
        Ok(s) => s == "true",
        _ => false
    }
}

macro_rules! trace {
    ($err:expr) => {
        println!("[ParseError][TreeBuilding]: {}", $err);
    }
}

macro_rules! emit_error {
    ($err:expr) => {
        if is_trace() {
            trace!($err)
        }
    }
}

macro_rules! match_any {
    ($target:ident, $($cmp:expr), *) => {
        $($target == $cmp)||*
    };
}

pub struct TreeBuilder {
    // the tokenizer
    tokenizer: Tokenizer,

    // stack of open elements as mentioned in specs
    open_elements: StackOfOpenElements,

    // current insertion mode
    insert_mode: InsertMode,

    // the insert mode that the builder will return
    original_insert_mode: Option<InsertMode>,

    // the result document
    document: NodeRef,

    // enable or disable foster parenting
    foster_parenting: bool,

    // element pointer to head element
    head_pointer: Option<NodeRef>,

    // is scripting enable?
    scripting: bool,

    // list of active formatting elements
    active_formatting_elements: ListOfActiveFormattingElements,

    // framset ok flag
    frameset_ok: bool,

    // stack of template insert mode to parse nested template
    stack_of_template_insert_mode: Vec<InsertMode>
}

pub struct AdjustedInsertionLocation {
    pub parent: NodeRef,
    pub insert_before_sibling: Option<NodeRef>
}

enum TextOnlyElementParsingAlgo {
    GenericRawText,
    GenericRCDataElement
}

#[derive(Debug, PartialEq)]
pub enum ProcessingResult {
    Continue,
    Stop
}

fn is_whitespace(c: char) -> bool {
    match c {
        '\t' | '\n' | '\x0C' | ' ' => true,
        _ => false
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
            original_insert_mode: None,
            scripting: false,
            active_formatting_elements: ListOfActiveFormattingElements::new(),
            frameset_ok: true,
            stack_of_template_insert_mode: Vec::new()
        }
    }

    pub fn run(&mut self) {
        loop {
            let token = self.tokenizer.next_token();
            if let Token::EOF = token {
                break
            }
            match self.process(token) {
                ProcessingResult::Continue => {},
                ProcessingResult::Stop => {
                    break
                }
            }
        }
    }

    pub fn process(&mut self, token: Token) -> ProcessingResult {
        match self.insert_mode {
            InsertMode::Initial => self.handle_initial(token),
            InsertMode::BeforeHtml => self.handle_before_html(token),
            InsertMode::BeforeHead => self.handle_before_head(token),
            InsertMode::InHead => self.handle_in_head(token),
            InsertMode::InHeadNoScript => self.handle_in_head_no_script(token),
            _ => unimplemented!()
        }
    }

    pub fn get_document(&self) -> NodeRef {
        self.document.clone()
    }

    fn which_quirks_mode(&self, token: Token) -> QuirksMode {
        if let Token::DOCTYPE {
            name,
            force_quirks,
            ..
        } = token {
            // TODO: Implement full stpecs detection
            if force_quirks || name.unwrap_or_default() != "html" {
                return QuirksMode::Quirks;
            }
        }
        QuirksMode::NoQuirks
    }

    fn switch_to(&mut self, mode: InsertMode) {
        self.insert_mode = mode;
    }

    fn create_element(&self, tag_token: Token) -> NodeRef {
        let (tag_name, attributes) = if let Token::Tag { tag_name, attributes, .. } = tag_token {
            (tag_name, attributes)
        } else {
            ("".to_string(), Vec::new())
        };
        let element_ref = create_element(self.document.clone().downgrade(), &tag_name);
        if let Some(element) = element_ref.borrow_mut().as_any_mut().downcast_mut::<Element>() {
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
            self_closing_acknowledged: false
        })
    }

    fn get_appropriate_place_for_inserting_a_node(&self) -> AdjustedInsertionLocation {
        let target = self.open_elements.current_node().unwrap();
        
        // TODO: implement full specs
        return AdjustedInsertionLocation {
            parent: target.clone(),
            insert_before_sibling: target.borrow().as_node().last_child()
        };
    }

    fn insert_html_element(&mut self, token: Token) -> NodeRef {
        let insert_position = self.get_appropriate_place_for_inserting_a_node();
        let element = self.create_element(token);
        let return_ref = element.clone();
        
        // TODO: check if location is possible to insert node (Idk why so we just leave it for now)
        self.open_elements.push(element.clone());
        Node::insert_before(insert_position.parent, element, insert_position.insert_before_sibling);
        return_ref
    }

    fn insert_character(&mut self, ch: char) {
        let insert_position = self.get_appropriate_place_for_inserting_a_node();
        if insert_position.parent.borrow().as_any().downcast_ref::<Document>().is_some() {
            return
        }
        if let Some(sibling) = insert_position.insert_before_sibling.clone() {
            if let Some(text) = sibling.borrow_mut().as_any_mut().downcast_mut::<Text>() {
                text.character_data.append_data(&ch.to_string());
                return
            }
        }
        let text = NodeRef::new(Text::new(ch.to_string()));
        Node::insert_before(insert_position.parent, text, insert_position.insert_before_sibling);
    }

    fn insert_comment(&mut self, data: String) {
        let insert_position = self.get_appropriate_place_for_inserting_a_node();
        let comment = NodeRef::new(Comment::new(data));
        Node::insert_before(insert_position.parent, comment, insert_position.insert_before_sibling);
    }

    fn handle_text_only_element(&mut self, token: Token, algorithm: TextOnlyElementParsingAlgo) {
        self.insert_html_element(token);
        match algorithm {
            TextOnlyElementParsingAlgo::GenericRawText => {
                self.tokenizer.switch_to(State::RAWTEXT);
            },
            TextOnlyElementParsingAlgo::GenericRCDataElement => {
                self.tokenizer.switch_to(State::RCDATA);
            }
        }
        self.original_insert_mode = Some(self.insert_mode.clone());
        self.switch_to(InsertMode::Text);
    }

    fn generate_all_implied_end_tags_thoroughly(&mut self) {
        while let Some(node) = self.open_elements.current_node() {
            let element = node.borrow().as_element().unwrap();
            let tag_name = element.tag_name();
            if match_any!(tag_name, "caption", "colgroup", "dd", "dt", "li", "optgroup", "option", "p", "rb", "rt", "rtc", "tbody", "td", "tfoot", "th", "thead", "tr") {
                self.open_elements.pop();
            }
            else {
                break
            }
        }
    }

    fn reset_insertion_mode_appropriately(&mut self) {
        unimplemented!(); // Do this next!
    }
}

impl TreeBuilder {
    fn handle_initial(&mut self, token: Token) -> ProcessingResult {
        match token.clone() {
            Token::Character(c) if is_whitespace(c) => ProcessingResult::Continue,
            Token::Comment(data) => {
                let comment = NodeRef::new(Comment::new(data));
                Node::append_child(self.document.clone(), comment);
                ProcessingResult::Continue
            }
            Token::DOCTYPE { name, public_identifier, system_identifier, .. } => {
                let name = name.unwrap_or_default();

                let error = match (name.as_str(), public_identifier.clone(), system_identifier.clone()) {
                    ("html", None, None)                                  => false,
                    ("html", None, Some(c)) if c == "about:legacy-compat" => false,
                    _ => true
                };

                if error {
                    emit_error!("Bad doctype");
                }

                let doctype = DocumentType::new(name, public_identifier, system_identifier);
                
                if let Some(doc) = self.document.borrow_mut().as_any_mut().downcast_mut::<Document>() {
                    doc.set_doctype(doctype);
                    doc.set_mode(self.which_quirks_mode(token));
                }

                self.switch_to(InsertMode::BeforeHtml);

                ProcessingResult::Continue
            }
            _ => {
                emit_error!("Bad token");
                self.switch_to(InsertMode::BeforeHtml);
                self.process(token)
            }
        }
    }

    fn handle_before_html(&mut self, token: Token) -> ProcessingResult {
        fn anything_else(this: &mut TreeBuilder, token: Token) -> ProcessingResult {
            let element = this.create_element_from_tag_name("html");
            Node::append_child(this.document.clone(), element.clone());
            this.open_elements.push(element.clone());
            // TODO: Implement additional steps in specs
            this.switch_to(InsertMode::BeforeHead);
            this.process(token.clone())
        }

        match token.clone() {
            Token::DOCTYPE { .. } => {
                emit_error!("Unexpected doctype");
                ProcessingResult::Continue
            }
            Token::Comment(data) => {
                let comment = NodeRef::new(Comment::new(data));
                Node::append_child(self.document.clone(), comment);
                ProcessingResult::Continue
            }
            Token::Character(c) if is_whitespace(c) => ProcessingResult::Continue,
            Token::Tag { tag_name, is_end_tag, .. } => {
                if tag_name == "html" && !is_end_tag {
                    let element = self.create_element(token);
                    Node::append_child(self.document.clone(), element.clone());
                    self.open_elements.push(element.clone());
                    // TODO: implement additional steps in specs
                    self.switch_to(InsertMode::BeforeHead);
                    ProcessingResult::Continue
                } else if match_any!(tag_name, "head", "body", "html", "br") && is_end_tag {
                    anything_else(self, token)
                } else {
                    if is_end_tag {
                        emit_error!("Invalid end tag");
                        return ProcessingResult::Continue;
                    }
                    anything_else(self, token)
                }
            }
            _ => anything_else(self, token)
        }
    }

    fn handle_before_head(&mut self, token: Token) -> ProcessingResult {
        fn anything_else(this: &mut TreeBuilder, token: Token) -> ProcessingResult {
            let head_element = this.insert_html_element(Token::Tag {
                tag_name: "head".to_owned(),
                attributes: Vec::new(),
                is_end_tag: false,
                self_closing: false,
                self_closing_acknowledged: false
            });
            this.head_pointer = Some(head_element.clone());
            this.switch_to(InsertMode::InHead);
            this.process(token)
        }
        match token.clone() {
            Token::Character(c) if is_whitespace(c) => ProcessingResult::Continue,
            Token::Comment(data) => {
                self.insert_comment(data);
                ProcessingResult::Continue
            }
            Token::DOCTYPE { .. } => {
                emit_error!("Unexpected doctype");
                ProcessingResult::Continue
            }
            Token::Tag { tag_name, is_end_tag, .. } => {
                if !is_end_tag && tag_name == "html" {
                    return self.handle_in_body(token);
                }
                if !is_end_tag && tag_name == "head" {
                    let head_element = self.insert_html_element(token);
                    self.head_pointer = Some(head_element);
                    self.switch_to(InsertMode::InHead);
                    return ProcessingResult::Continue
                }
                if is_end_tag && match_any!(tag_name, "head", "body", "html", "br") {
                    return anything_else(self, token);
                }
                if is_end_tag {
                    emit_error!("Invalid end tag");
                    return ProcessingResult::Continue
                }
                anything_else(self, token)
            }
            _ => anything_else(self, token)
        }
    }

    fn handle_in_head(&mut self, token: Token) -> ProcessingResult {
        match token.clone() {
            Token::Character(c) if is_whitespace(c) => {
                self.insert_character(c);
                ProcessingResult::Continue
            }
            Token::Comment(data) => {
                self.insert_comment(data);
                ProcessingResult::Continue
            }
            Token::DOCTYPE { .. } => {
                emit_error!("Unexpected doctype");
                ProcessingResult::Continue
            }
            Token::Tag { tag_name, is_end_tag, .. } => {
                if !is_end_tag && tag_name == "html" {
                    return self.handle_in_body(token);
                }
                if !is_end_tag && match_any!(tag_name, "base", "basefont", "bgsound", "link") {
                    self.insert_html_element(token);
                    self.open_elements.pop();
                    token.acknowledge_self_closing_if_set();
                    return ProcessingResult::Continue;
                }
                if !is_end_tag && tag_name == "meta" {
                    self.insert_html_element(token);
                    self.open_elements.pop();
                    token.acknowledge_self_closing_if_set();
                    return ProcessingResult::Continue;
                }
                if !is_end_tag && tag_name == "title" {
                    self.handle_text_only_element(token, TextOnlyElementParsingAlgo::GenericRCDataElement);
                    return ProcessingResult::Continue;
                }
                if !is_end_tag && tag_name == "noscript" && self.scripting {
                    self.handle_text_only_element(token, TextOnlyElementParsingAlgo::GenericRawText);
                    return ProcessingResult::Continue;
                }
                if !is_end_tag && match_any!(tag_name, "noframes", "style") {
                    self.handle_text_only_element(token, TextOnlyElementParsingAlgo::GenericRawText);
                    return ProcessingResult::Continue;
                }
                if !is_end_tag && tag_name == "noscript" && !self.scripting {
                    self.insert_html_element(token);
                    self.switch_to(InsertMode::InHeadNoScript);
                    return ProcessingResult::Continue;
                }
                if !is_end_tag && tag_name == "script" {
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
                        insert_position.insert_before_sibling
                    );
                    self.open_elements.push(element.clone());
                    self.tokenizer.switch_to(State::ScriptData);
                    self.original_insert_mode = Some(self.insert_mode.clone());
                    self.switch_to(InsertMode::Text);
                    return ProcessingResult::Continue;
                }

                if is_end_tag && tag_name == "head" {
                    self.open_elements.pop();
                    self.switch_to(InsertMode::AfterHead);
                    return ProcessingResult::Continue;
                }

                if is_end_tag && match_any!(tag_name, "body", "html", "br") {
                    self.open_elements.pop();
                    self.switch_to(InsertMode::AfterHead);
                    return self.process(token);
                }

                if !is_end_tag && tag_name == "template" {
                    self.insert_html_element(token);
                    self.active_formatting_elements.add_marker();
                    self.frameset_ok = false;
                    self.switch_to(InsertMode::InTemplate);
                    self.stack_of_template_insert_mode.push(InsertMode::InTemplate);
                    return ProcessingResult::Continue;
                }

                if is_end_tag && tag_name == "template" {
                    if !self.open_elements.contains("template") {
                        emit_error!("No template tag found");
                        return ProcessingResult::Continue;
                    }

                    self.generate_all_implied_end_tags_thoroughly();

                    if let Some(node) = self.open_elements.current_node() {
                        let element = node.borrow().as_element().unwrap();
                        if element.tag_name() != "template" {
                            emit_error!("Expected current node to be template");
                        }
                    }

                    self.open_elements.pop_until("template");
                    self.active_formatting_elements.clear_up_to_last_marker();
                    self.stack_of_template_insert_mode.pop();
                    self.reset_insertion_mode_appropriately();
                }

                if (!is_end_tag && tag_name == "head") || !is_end_tag {
                    emit_error!("Unexpected tag token");
                    return ProcessingResult::Continue;
                }

                self.open_elements.pop();
                self.switch_to(InsertMode::AfterHead);
                self.process(token)
            }
            _ => {
                self.open_elements.pop();
                self.switch_to(InsertMode::AfterHead);
                self.process(token)
            }
        }
    }

    fn handle_in_head_no_script(&mut self, token: Token) -> ProcessingResult {
        match token {
            Token::DOCTYPE { .. } => {
                emit_error!("Unexpected doctype");
                ProcessingResult::Continue
            }
            Token::Tag { is_end_tag, tag_name, .. } => {
                if !is_end_tag && tag_name == "html" {
                    return self.handle_in_body(token);
                }

                if is_end_tag && tag_name == "noscript" {
                    self.open_elements.pop();
                    self.switch_to(InsertMode::InHead);
                    return ProcessingResult::Continue;
                }

                if !is_end_tag && match_any!(tag_name, "basefont", "bgsound", "link", "meta", "noframes", "style") {
                    return self.handle_in_head(token);
                }

                if is_end_tag && tag_name == "br" {
                    emit_error!("Unexpected br");
                    self.open_elements.pop();
                    self.switch_to(InsertMode::InHead);
                    return self.process(token);
                }

                if (!is_end_tag && match_any!(tag_name, "head", "noscript")) || is_end_tag {
                    emit_error!("Unexpected tag token");
                    return ProcessingResult::Continue;
                }

                emit_error!("Unexpected tag token");
                self.open_elements.pop();
                self.switch_to(InsertMode::InHead);
                return self.process(token);
            }
            Token::Character(c) if is_whitespace(c) => self.handle_in_head(token),
            Token::Comment(_) => self.handle_in_head(token),
            _ => {
                emit_error!("Unexpected token");
                self.open_elements.pop();
                self.switch_to(InsertMode::InHead);
                return self.process(token);
            }
        }
    }

    fn handle_in_body(&mut self, token: Token) -> ProcessingResult {
        ProcessingResult::Continue
    }
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

        println!("{:#?}", tree_builder.get_document().borrow().as_node());

        if let Some(child) = tree_builder.get_document().borrow().as_node().first_child() {
            if let Some(comment) = child.borrow().as_any().downcast_ref::<Comment>() {
                assert_ne!(comment.get_data(), "this is a test");
            } else {
                panic!("First child is not a comment");
            }
        } else {
            panic!("There is no first child");
        }

        if let Some(child) = tree_builder.get_document().borrow().as_node().last_child() {
            if let Some(comment) = child.borrow().as_any().downcast_ref::<Comment>() {
                assert_ne!(comment.get_data(), "this is a test");
            } else {
                panic!("Last child is not a comment");
            }
        } else {
            panic!("There is no last child");
        }
    }
}
