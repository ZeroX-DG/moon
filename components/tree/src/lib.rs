use std::rc::{Weak, Rc};
use std::cell::RefCell;
use std::ops::Deref;

#[derive(Debug)]
pub struct TreeNodeRef<T>(Rc<RefCell<T>>);

#[derive(Debug)]
pub struct TreeNodeWeakRef<T>(Weak<RefCell<T>>);

impl<T> Deref for TreeNodeRef<T> {
    type Target = RefCell<T>;

    fn deref(&self) -> &RefCell<T> {
        &*self.0
    }
}

impl<T> Clone for TreeNodeWeakRef<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Clone for TreeNodeRef<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> PartialEq for TreeNodeRef<T> {
    fn eq(&self, other: &TreeNodeRef<T>) -> bool {
        self.as_ptr().eq(&other.as_ptr())
    }
}

impl<T> TreeNodeRef<T> {
    pub fn new(inner: T) -> Self {
        Self(Rc::new(RefCell::new(inner)))
    }

    pub fn downgrade(&self) -> TreeNodeWeakRef<T> {
        TreeNodeWeakRef(Rc::downgrade(&self.0))
    }
}

impl <T> TreeNodeWeakRef<T> {
    pub fn upgrade(&self) -> Option<TreeNodeRef<T>> {
        match self.0.upgrade() {
            Some(node) => Some(TreeNodeRef(node)),
            _ => None,
        }
    }
}
