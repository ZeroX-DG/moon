use style_types::{Property, Value};

pub type ExpandOutput = Option<Vec<(Property, Option<Value>)>>;

mod border;
mod border_color;
mod border_radius;
mod border_style;
mod border_width;
mod margin;
mod padding;

pub(crate) mod prelude {
    pub use super::border::expand_border;
    pub use super::border_color::expand_border_color;
    pub use super::border_radius::expand_border_radius;
    pub use super::border_style::expand_border_style;
    pub use super::border_width::expand_border_width;
    pub use super::margin::expand_margin;
    pub use super::padding::expand_padding;
    pub use super::ExpandOutput;
}
