use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum KernelMessage {
    LoadHTMLLocal(String),
    LoadCSSLocal(String),
    Exit,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RendererMessage {
    RePaint(Vec<u8>),
    ResourceNotFound(String),
    Exit,
}