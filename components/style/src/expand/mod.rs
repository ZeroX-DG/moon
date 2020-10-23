use crate::value_processing::{Property, Value};
pub type ExpandOutput = Option<Vec<(Property, Option<Value>)>>;

pub mod border;
pub mod border_color;
pub mod border_style;
pub mod border_width;
pub mod margin;
pub mod padding;
