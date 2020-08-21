use crate::node::{NodeRef, NodeInner};
use crate::nodes;

pub trait DocumentType {
    fn system_id(&self) -> String;
    fn public_id(&self) -> String;
    fn name(&self) -> String;
}

impl DocumentType for NodeRef {
    fn system_id(&self) -> String {
        let ref_self = self.borrow();
        if let NodeInner::DocumentType(doctype) = &*ref_self.inner.borrow() {
            let nodes::DocumentType { system_id, .. } = doctype;
            return system_id.clone()
        }
        panic!("Node is not document type")
    }

    fn public_id(&self) -> String {
        let ref_self = self.borrow();
        if let NodeInner::DocumentType(doctype) = &*ref_self.inner.borrow() {
            let nodes::DocumentType { public_id, .. } = doctype;
            return public_id.clone()
        }
        panic!("Node is not document type")
    }

    fn name(&self) -> String {
        let ref_self = self.borrow();
        if let NodeInner::DocumentType(doctype) = &*ref_self.inner.borrow() {
            let nodes::DocumentType { name, .. } = doctype;
            return name.clone()
        }
        panic!("Node is not document type")
    }
}
