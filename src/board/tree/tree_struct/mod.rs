use std::{fmt::Display, ptr::NonNull};

mod constructors;
mod read;
mod operations;
mod stacks;
mod test_funcs;
mod iteration;
mod io;

use super::Node;

pub struct Tree<V: Ord + Sized + Default> {
    sentinel: NonNull<Node<V>>,
}

impl<V: Ord + Sized + Default> Drop for Tree<V> {
    fn drop(&mut self) {
        self.clear();
        unsafe {
            let _ = Box::from_raw(self.sentinel.as_ptr());
        };
    }
}

unsafe impl<V: Send + Ord + ?Sized + Default> Send for Tree<V> {}
unsafe impl<V: Sync + Ord + ?Sized + Default> Sync for Tree<V> {}