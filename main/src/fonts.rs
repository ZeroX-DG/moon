use iced::Font;

pub const ICON_FONT: Font = iced::Font::External {
    name: "Icons",
    bytes: include_bytes!("../fonts/icofont.ttf")
};