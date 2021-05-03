use serde::{de::DeserializeOwned, Serialize};

pub trait Notification {
    type Params: DeserializeOwned + Serialize;
    const METHOD: &'static str;
}

pub enum Exit {}

impl Notification for Exit {
    type Params = ();
    const METHOD: &'static str = "exit";
}
