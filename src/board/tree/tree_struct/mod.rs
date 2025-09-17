use std::ptr::NonNull;

mod constructors;
mod read;
mod operations;
mod stacks;
mod test_funcs;
mod iteration;

use super::Node;

pub struct Tree<V: Ord + Sized + Default + Clone> {
    sentinel: NonNull<Node<V>>,
}

impl<V: Ord + Sized + Default + Clone> Drop for Tree<V> {
    fn drop(&mut self) {
        self.clear();
        unsafe {
            let _ = Box::from_raw(self.sentinel.as_ptr());
        };
    }
}

impl<V: Ord + Sized + Default + Clone> Clone for Tree<V> {
    fn clone(&self) -> Self {
        Self::from_tree(self)
    }
}

unsafe impl<V: Send + Ord + ?Sized + Default + Clone> Send for Tree<V> {}
unsafe impl<V: Sync + Ord + ?Sized + Default + Clone> Sync for Tree<V> {}