use serde::{Deserialize, Serialize};
use super::values::{Rect, Paint};

#[derive(Debug, Serialize, Deserialize)]
pub enum DisplayCommand {
    DrawRect(Rect, Paint),
}