use serde::{de::DeserializeOwned, Serialize};
use super::*;

pub trait Notification {
    type Params: DeserializeOwned + Serialize;
    const METHOD: &'static str;
}

pub enum Exit {}

impl Notification for Exit {
    type Params = ();
    const METHOD: &'static str = "exit";
}

pub enum Syn {}

impl Notification for Syn {
    type Params = SynParams;
    const METHOD: &'static str = "syn";
}

pub enum SynAck {}

impl Notification for SynAck {
    type Params = ();
    const METHOD: &'static str = "syn-ack";
}

pub enum Ack {}

impl Notification for Ack {
    type Params = SynParams;
    const METHOD: &'static str = "ack";
}

pub enum LoadFile {}

impl Notification for LoadFile {
    type Params = LoadFileContentParams;
    const METHOD: &'static str = "load-html";
}

