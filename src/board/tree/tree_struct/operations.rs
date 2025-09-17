use std::{cmp, ptr::NonNull};

use super::Node;
use super::Tree;
use super::stacks::*;

impl<V: Ord + Sized + Default + Clone> Tree<V> {
    pub fn insert(&mut self, val: V) -> bool {
        unsafe {
            if (*self.sentinel.as_ptr()).right.is_none() {
                let node = NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                    count: 1,
                    height: 1,
                    left: None,
                    right: None,
                    parent: Some(NonNull::new_unchecked(self.sentinel.as_ptr())),
                    is_left_child: false,
                    val: val,
                })));
                (*self.sentinel.as_ptr()).right = Some(node);
                return true;
            }

            // left is true, right is false
            let mut dir: bool;
            let mut parent = (*self.sentinel.as_ptr()).right.unwrap().as_ptr().clone();

            loop {
                match val.cmp(&(*parent).val) {
                    cmp::Ordering::Equal => {
                        return false;
                    }
                    cmp::Ordering::Greater => {
                        dir = false;
                        if (*parent).right.is_none() {
                            break;
                        }
                        parent = (*parent).right.unwrap().as_ptr().clone();
                    }
                    cmp::Ordering::Less => {
                        dir = true;
                        if (*parent).left.is_none() {
                            break;
                        }
                        parent = (*parent).left.unwrap().as_ptr().clone();
                    }
                };
            }

            let new_node = NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                count: 1,
                height: 1,
                left: None,
                right: None,
                parent: Some(NonNull::new_unchecked(parent)),
                is_left_child: dir,
                val: val,
            })));

            if dir {
                (*parent).left = Some(new_node);
            } else {
                (*parent).right = Some(new_node);
            }

            self.recursive_fix_up(parent);

            return true;
        }
    }

    pub fn remove(&mut self, val: V) -> bool {
        unsafe {
            if (*self.sentinel.as_ptr()).right.is_none() {
                return false;
            }
            let mut node = (*self.sentinel.as_ptr()).right.unwrap().as_ptr();

            loop {
                match val.cmp(&(*node).val) {
                    cmp::Ordering::Equal => {
                        break;
                    }
                    cmp::Ordering::Greater => {
                        if (*node).right.is_none() {
                            return false;
                        }
                        node = (*node).right.unwrap().as_ptr();
                    }
                    cmp::Ordering::Less => {
                        if (*node).left.is_none() {
                            return false;
                        }
                        node = (*node).left.unwrap().as_ptr();
                    }
                };
            }

            self.remove_node(node);

            return true;
        }
    }

    unsafe fn remove_node(&mut self, node: *mut Node<V>) -> Box<Node<V>> {
        unsafe {
            let parent = match (*node).parent {
                Some(v) => v.as_ptr(),
                None => panic!("Cannot remove sentinel node!"),
            };

            // if there's no left child, replace me with my right child
            if (*node).left.is_none() {
                if (*node).is_left_child {
                    (*parent).left = (*node).right
                } else {
                    (*parent).right = (*node).right
                }

                if let Some(child) = (*node).right.map(|ptr| ptr.as_ptr()) {
                    (*child).parent = (*node).parent;
                    (*child).is_left_child = (*node).is_left_child;
                }

                self.recursive_fix_up(parent);
                return Box::from_raw(node);
            } else if (*node).right.is_none() {
                // if theres a left child and no right child, replace me with my left child
                if (*node).is_left_child {
                    (*parent).left = (*node).left
                } else {
                    (*parent).right = (*node).left
                }

                let child = (*node).left.unwrap().as_ptr();
                (*child).parent = (*node).parent;
                (*child).is_left_child = (*node).is_left_child;

                self.recursive_fix_up(parent);
                return Box::from_raw(node);
            }

            /* If I have two children, then find the node "before" me in the tree.
            That node will have no right child, so I can recursively delete it.
            When I'm done, I'll swap out this node with that one. */
            let mut replace_ptr = (*node).left.unwrap().as_ptr();
            while (*replace_ptr).right.is_some() {
                replace_ptr = (*replace_ptr).right.unwrap().as_ptr();
            }

            let replace_node = Box::into_raw(self.remove_node(replace_ptr));

            if (*node).is_left_child {
                (*parent).left = Some(NonNull::new_unchecked(replace_node));
            } else {
                (*parent).right = Some(NonNull::new_unchecked(replace_node));
            }

            (*replace_node).parent = (*node).parent;

            (*replace_node).left = (*node).left;
            if let Some(child) = (*replace_node).left.map(|ptr| ptr.as_ptr()) {
                (*child).parent = Some(NonNull::new_unchecked(replace_node));
            }

            (*replace_node).right = (*node).right;
            if let Some(child) = (*replace_node).right.map(|ptr| ptr.as_ptr()) {
                (*child).parent = Some(NonNull::new_unchecked(replace_node));
            }

            self.recursive_fix_up(replace_node);
            return Box::from_raw(node);
        }
    }

    unsafe fn recursive_fix_up(&mut self, node: *mut Node<V>) {
        let mut node = node;
        unsafe {
            while let Some(next_node) = (*node).parent.map(|ptr| ptr.as_ptr().clone()) {
                if Node::is_imbalanced(node) {
                    Node::fix_imbalance(node);
                } else {
                    Node::fix(node);
                }

                node = next_node;
            }
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            let mut stack: Vec<StackEntry<V>> = Vec::with_capacity(self.height());

            if self.is_empty() {
                return;
            }

            stack.push(StackEntry {
                ptr: (*self.sentinel.as_ptr()).right.unwrap().as_ptr(),
                state: StackState::Left,
            });
            let mut last = stack.last_mut().unwrap();

            loop {
                match last.state {
                    StackState::Left => {
                        last.state = StackState::Right;
                        if let Some(child) = (*last.ptr).left {
                            let entry = StackEntry {
                                ptr: child.clone().as_ptr(),
                                state: StackState::Left,
                            };
                            stack.push(entry);
                            last = stack.last_mut().unwrap();
                        }
                    }
                    StackState::Right => {
                        last.state = StackState::Handle;
                        if let Some(child) = (*last.ptr).right {
                            let entry = StackEntry {
                                ptr: child.clone().as_ptr(),
                                state: StackState::Left,
                            };
                            stack.push(entry);
                            last = stack.last_mut().unwrap();
                        }
                    }
                    StackState::Handle => {
                        let _ = Box::from_raw(last.ptr);
                        stack.pop();
                        last = match stack.last_mut() {
                            Some(v) => v,
                            None => {
                                break;
                            }
                        };
                    }
                };
            }

            (*self.sentinel.as_ptr()).right = None;
        }
    }
}