use serde::{Deserialize, Serialize};
use style::value::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }
}

pub fn style_color_to_paint_color(style_color: &Value) -> Option<Color> {
    let color = match style_color {
        Value::Color(c) => c,
        _ => return None,
    };

    match color {
        style::values::color::Color::Rgba(r, g, b, a) => {
            let alpha: u8 = a.as_u8();
            Some(Color {
                r: r.as_u8(),
                g: g.as_u8(),
                b: b.as_u8(),
                a: alpha,
            })
        }
        _ => None,
    }
}
