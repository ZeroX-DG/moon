use serde::{de::DeserializeOwned, Serialize};
use super::*;

pub trait Request {
    type Params: DeserializeOwned + Serialize;
    type Result: DeserializeOwned + Serialize;
    const METHOD: &'static str;
}

pub enum GetRenderedBitmap {}

impl Request for GetRenderedBitmap {
    type Params = ();
    type Result = RenderedBitmap;
    const METHOD: &'static str = "getRenderedBitmap";
}

