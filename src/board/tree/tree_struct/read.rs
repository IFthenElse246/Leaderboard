use std::cmp;

use crate::board::tree::node::Node;

use super::Tree;

impl<V: Ord + Sized + Default> Tree<V> {
    pub fn contains(&self, val: &V) -> bool {
        unsafe {
            let mut parent = match (*self.sentinel.as_ptr()).right {
                None => {
                    return false;
                }
                Some(ptr) => ptr.as_ptr(),
            };

            loop {
                match val.cmp(&(*parent).val) {
                    cmp::Ordering::Equal => {
                        return true;
                    }
                    cmp::Ordering::Greater => {
                        if (*parent).right.is_none() {
                            return false;
                        }
                        parent = (*parent).right.unwrap().as_ptr();
                    }
                    cmp::Ordering::Less => {
                        if (*parent).left.is_none() {
                            return false;
                        }
                        parent = (*parent).left.unwrap().as_ptr();
                    }
                };
            }
        }
    }

    pub fn index_of(&self, val: &V) -> (usize, bool) {
        let mut ind: usize = 0;
        unsafe {
            let mut parent = match (*self.sentinel.as_ptr()).right {
                None => {
                    return (0, false);
                }
                Some(ptr) => ptr.as_ptr(),
            };

            loop {
                match val.cmp(&(*parent).val) {
                    cmp::Ordering::Equal => {
                        ind += Node::get_right_count(parent);
                        return (ind, true);
                    }
                    cmp::Ordering::Greater => {
                        if (*parent).right.is_none() {
                            return (ind, false);
                        }
                        parent = (*parent).right.unwrap().as_ptr();
                    }
                    cmp::Ordering::Less => {
                        ind += 1 + Node::get_right_count(parent);
                        if (*parent).left.is_none() {
                            return (ind, false);
                        }
                        parent = (*parent).left.unwrap().as_ptr();
                    }
                };
            }
        }
    }

    pub(super) fn node_at_index(&self, ind: usize) -> Option<*mut Node<V>> {
        let mut amount = ind;
        unsafe {
            let mut parent = (*self.sentinel.as_ptr()).right?.as_ptr();

            loop {
                let right = Node::get_right_count(parent);
                match amount.cmp(&right) {
                    cmp::Ordering::Equal => {
                        return Some(parent);
                    }
                    cmp::Ordering::Less => {
                        if (*parent).right.is_none() {
                            return None;
                        }
                        parent = (*parent).right.unwrap().as_ptr();
                    }
                    cmp::Ordering::Greater => {
                        if (*parent).left.is_none() {
                            return None;
                        }
                        amount -= right + 1;
                        parent = (*parent).left.unwrap().as_ptr();
                    }
                };
            }
        }
    }

    pub fn at_index<'l>(&'l self, ind: usize) -> Option<&'l V> {
        unsafe { self.node_at_index(ind).map(|v| &(*v).val) }
    }

    pub fn is_empty(&self) -> bool {
        unsafe { (*self.sentinel.as_ptr()).right.is_none() }
    }

    pub fn len(&self) -> usize {
        unsafe { Node::get_right_count(self.sentinel.as_ptr()) }
    }

    pub fn height(&self) -> usize {
        unsafe {
            match (*self.sentinel.as_ptr()).right {
                Some(ptr) => (*ptr.as_ptr()).height,
                None => 0,
            }
        }
    }

    // pub fn before<'a, 'b>(&'a self, count: usize, last_exclusive: &'b V) -> Vec<&'a V> {
    //     if self.is_empty() {
    //         return Vec::new();
    //     }

    //     let mut stack: Vec<StackEntry<V>> = Vec::with_capacity(self.height());
    //     let mut ret: Vec<&'a V> = Vec::with_capacity(count);

    //     loop {

    //     }
    // }
}
