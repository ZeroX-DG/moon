use css::{parser::structs::ComponentValue, tokenizer::token::Token};

use crate::property::Property;

use super::values::prelude::*;

macro_rules! parse_value {
    (Auto; $tokens:ident) => {{
        if parse_keyword($tokens, "auto") {
            Some(Value::Auto)
        } else {
            None
        }
    }};
    (Inherit; $tokens:ident) => {{
        if parse_keyword($tokens, "inherit") {
            Some(Value::Inherit)
        } else {
            None
        }
    }};
    (Initial; $tokens:ident) => {{
        if parse_keyword($tokens, "initial") {
            Some(Value::Initial)
        } else {
            None
        }
    }};
    (Unset; $tokens:ident) => {{
        if parse_keyword($tokens, "unset") {
            Some(Value::Unset)
        } else {
            None
        }
    }};
    ($value:ident; $tokens:ident) => {{
        if let Some(value) = $value::parse($tokens) {
            Some(Value::$value(value))
        } else {
            None
        }
    }};
    ($value:ident | $($remain:ident)|+; $tokens:ident) => {{
        let value = parse_value!($value; $tokens);
        if value.is_some() {
            return value;
        }
        parse_value!($($remain)|+; $tokens)
    }};
}

/// CSS property value
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Value {
    Color(Color),
    Display(Display),
    Length(Length),
    Percentage(Percentage),
    BorderStyle(BorderStyle),
    BorderWidth(BorderWidth),
    Float(Float),
    Position(Position),
    Direction(Direction),
    BorderRadius(BorderRadius),
    Auto,
    Inherit,
    Initial,
    Unset,
}

impl Value {
    pub fn parse(property: &Property, tokens: &[ComponentValue]) -> Option<Self> {
        match property {
            Property::BackgroundColor => parse_value!(
                Color | Inherit | Initial | Unset;
                tokens
            ),
            Property::Color => parse_value!(
                Color | Inherit | Initial | Unset;
                tokens
            ),
            Property::Display => parse_value!(
                Display | Inherit | Initial | Unset;
                tokens
            ),
            Property::Width => parse_value!(
                Length | Percentage | Auto | Inherit | Initial | Unset;
                tokens
            ),
            Property::Height => parse_value!(
                Length | Percentage | Auto | Inherit | Initial | Unset;
                tokens
            ),
            Property::MarginTop => parse_value!(
                Length | Percentage | Auto | Inherit | Initial | Unset;
                tokens
            ),
            Property::MarginRight => parse_value!(
                Length | Percentage | Auto | Inherit | Initial | Unset;
                tokens
            ),
            Property::MarginBottom => parse_value!(
                Length | Percentage | Auto | Inherit | Initial | Unset;
                tokens
            ),
            Property::MarginLeft => parse_value!(
                Length | Percentage | Auto | Inherit | Initial | Unset;
                tokens
            ),
            Property::PaddingTop => parse_value!(
                Length | Percentage | Inherit | Initial | Unset;
                tokens
            ),
            Property::PaddingRight => parse_value!(
                Length | Percentage | Inherit | Initial | Unset;
                tokens
            ),
            Property::PaddingBottom => parse_value!(
                Length | Percentage | Inherit | Initial | Unset;
                tokens
            ),
            Property::PaddingLeft => parse_value!(
                Length | Percentage | Inherit | Initial | Unset;
                tokens
            ),
            Property::BorderTopStyle => parse_value!(
                BorderStyle | Inherit | Initial | Unset;
                tokens
            ),
            Property::BorderRightStyle => parse_value!(
                BorderStyle | Inherit | Initial | Unset;
                tokens
            ),
            Property::BorderBottomStyle => parse_value!(
                BorderStyle | Inherit | Initial | Unset;
                tokens
            ),
            Property::BorderLeftStyle => parse_value!(
                BorderStyle | Inherit | Initial | Unset;
                tokens
            ),
            Property::BorderTopWidth => parse_value!(
                BorderWidth | Length | Inherit | Initial | Unset;
                tokens
            ),
            Property::BorderRightWidth => parse_value!(
                BorderWidth | Length | Inherit | Initial | Unset;
                tokens
            ),
            Property::BorderBottomWidth => parse_value!(
                BorderWidth | Length | Inherit | Initial | Unset;
                tokens
            ),
            Property::BorderLeftWidth => parse_value!(
                BorderWidth | Length | Inherit | Initial | Unset;
                tokens
            ),
            Property::BorderTopColor => parse_value!(
                Color | Inherit | Initial | Unset;
                tokens
            ),
            Property::BorderRightColor => parse_value!(
                Color | Inherit | Initial | Unset;
                tokens
            ),
            Property::BorderBottomColor => parse_value!(
                Color | Inherit | Initial | Unset;
                tokens
            ),
            Property::BorderLeftColor => parse_value!(
                Color | Inherit | Initial | Unset;
                tokens
            ),
            Property::Float => parse_value!(
                Float | Inherit | Initial | Unset;
                tokens
            ),
            Property::Position => parse_value!(
                Position | Inherit | Initial | Unset;
                tokens
            ),
            Property::Top => parse_value!(
                Length | Percentage | Auto | Inherit | Initial | Unset;
                tokens
            ),
            Property::Right => parse_value!(
                Length | Percentage | Auto | Inherit | Initial | Unset;
                tokens
            ),
            Property::Bottom => parse_value!(
                Length | Percentage | Auto | Inherit | Initial | Unset;
                tokens
            ),
            Property::Left => parse_value!(
                Length | Percentage | Auto | Inherit | Initial | Unset;
                tokens
            ),
            Property::Direction => parse_value!(
                Direction | Inherit | Initial | Unset;
                tokens
            ),
            Property::BorderTopLeftRadius => parse_value!(
                BorderRadius | Inherit | Initial | Unset;
                tokens
            ),
            Property::BorderTopRightRadius => parse_value!(
                BorderRadius | Inherit | Initial | Unset;
                tokens
            ),
            Property::BorderBottomLeftRadius => parse_value!(
                BorderRadius | Inherit | Initial | Unset;
                tokens
            ),
            Property::BorderBottomRightRadius => parse_value!(
                BorderRadius | Inherit | Initial | Unset;
                tokens
            ),
        }
    }

    pub fn initial(property: &Property) -> Value {
        match property {
            Property::BackgroundColor => Value::Color(Color::transparent()),
            Property::Color => Value::Color(Color::black()),
            Property::Display => Value::Display(Display::new_inline()),
            Property::Width => Value::Auto,
            Property::Height => Value::Auto,
            Property::MarginTop => Value::Length(Length::zero()),
            Property::MarginRight => Value::Length(Length::zero()),
            Property::MarginBottom => Value::Length(Length::zero()),
            Property::MarginLeft => Value::Length(Length::zero()),
            Property::PaddingTop => Value::Length(Length::zero()),
            Property::PaddingRight => Value::Length(Length::zero()),
            Property::PaddingBottom => Value::Length(Length::zero()),
            Property::PaddingLeft => Value::Length(Length::zero()),
            Property::BorderTopStyle => Value::BorderStyle(BorderStyle::None),
            Property::BorderRightStyle => Value::BorderStyle(BorderStyle::None),
            Property::BorderBottomStyle => Value::BorderStyle(BorderStyle::None),
            Property::BorderLeftStyle => Value::BorderStyle(BorderStyle::None),
            Property::BorderTopWidth => Value::BorderWidth(BorderWidth::Medium),
            Property::BorderRightWidth => Value::BorderWidth(BorderWidth::Medium),
            Property::BorderBottomWidth => Value::BorderWidth(BorderWidth::Medium),
            Property::BorderLeftWidth => Value::BorderWidth(BorderWidth::Medium),
            Property::BorderTopColor => Value::Color(Color::black()),
            Property::BorderRightColor => Value::Color(Color::black()),
            Property::BorderBottomColor => Value::Color(Color::black()),
            Property::BorderLeftColor => Value::Color(Color::black()),
            Property::Float => Value::Float(Float::None),
            Property::Position => Value::Position(Position::Static),
            Property::Left => Value::Auto,
            Property::Right => Value::Auto,
            Property::Bottom => Value::Auto,
            Property::Top => Value::Auto,
            Property::Direction => Value::Direction(Direction::Ltr),
            Property::BorderTopLeftRadius => Value::BorderRadius(BorderRadius::zero()),
            Property::BorderTopRightRadius => Value::BorderRadius(BorderRadius::zero()),
            Property::BorderBottomLeftRadius => Value::BorderRadius(BorderRadius::zero()),
            Property::BorderBottomRightRadius => Value::BorderRadius(BorderRadius::zero()),
        }
    }
}

fn parse_keyword(tokens: &[ComponentValue], keyword: &str) -> bool {
    match tokens.iter().next() {
        Some(ComponentValue::PerservedToken(Token::Ident(word))) => {
            word.eq_ignore_ascii_case(keyword)
        }
        _ => false,
    }
}
