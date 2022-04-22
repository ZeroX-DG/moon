use strum_macros::*;

/// CSS property name
#[derive(Debug, Clone, Hash, Eq, PartialEq, EnumIter)]
pub enum Property {
    BackgroundColor,
    Color,
    Display,
    Width,
    Height,
    MarginTop,
    MarginRight,
    MarginBottom,
    MarginLeft,
    PaddingTop,
    PaddingRight,
    PaddingBottom,
    PaddingLeft,
    BorderTopWidth,
    BorderRightWidth,
    BorderBottomWidth,
    BorderLeftWidth,
    BorderBottomStyle,
    BorderLeftStyle,
    BorderRightStyle,
    BorderTopStyle,
    BorderTopColor,
    BorderRightColor,
    BorderBottomColor,
    BorderLeftColor,
    BorderTopLeftRadius,
    BorderTopRightRadius,
    BorderBottomLeftRadius,
    BorderBottomRightRadius,
    Position,
    Float,
    Left,
    Right,
    Top,
    Bottom,
    Direction,
    FontSize,
    TextAlign,
}

impl Property {
    pub fn parse(property: &str) -> Option<Self> {
        match property {
            "background-color" => Some(Property::BackgroundColor),
            "color" => Some(Property::Color),
            "display" => Some(Property::Display),
            "width" => Some(Property::Width),
            "height" => Some(Property::Height),
            "margin-top" => Some(Property::MarginTop),
            "margin-right" => Some(Property::MarginRight),
            "margin-bottom" => Some(Property::MarginBottom),
            "margin-left" => Some(Property::MarginLeft),
            "padding-top" => Some(Property::PaddingTop),
            "padding-right" => Some(Property::PaddingRight),
            "padding-bottom" => Some(Property::PaddingBottom),
            "padding-left" => Some(Property::PaddingLeft),
            "float" => Some(Property::Float),
            "position" => Some(Property::Position),
            "left" => Some(Property::Left),
            "right" => Some(Property::Right),
            "top" => Some(Property::Top),
            "bottom" => Some(Property::Bottom),
            "direction" => Some(Property::Direction),
            "border-top-left-radius" => Some(Property::BorderTopLeftRadius),
            "border-top-right-radius" => Some(Property::BorderTopRightRadius),
            "border-bottom-left-radius" => Some(Property::BorderBottomLeftRadius),
            "border-bottom-right-radius" => Some(Property::BorderBottomRightRadius),
            "font-size" => Some(Property::FontSize),
            "margin-block-start" => Some(Property::MarginTop),
            "margin-block-end" => Some(Property::MarginBottom),
            "text-align" => Some(Property::TextAlign),
            _ => {
                log::debug!("Unsupported CSS property: {}", property);
                None
            }
        }
    }
}
