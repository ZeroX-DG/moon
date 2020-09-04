use super::CharacterData;
use crate::node::NodeRef;

pub trait Text : CharacterData {}

impl Text for NodeRef {}

impl CharacterData for NodeRef {
    fn append_data(&mut self, data: String) {
        
    }
    fn insert_data(&mut self, offset: usize, data: String) {
    }
    fn delete_data(&mut self, offset: usize, count: usize) {
        
    }
    fn replace_data(&mut self, offset: usize, count: usize, data: String) {
        
    }
}
