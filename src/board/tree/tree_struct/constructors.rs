use std::ptr::NonNull;

use super::Node;
use super::Tree;
use super::stacks::*;

impl<V: Ord + Sized + Default> Tree<V> {
    pub fn new() -> Self {
        unsafe {
            Self {
                sentinel: NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                    parent: None,
                    left: None,
                    right: None,
                    count: 0,
                    height: 0,
                    is_left_child: false,
                    val: V::default(),
                }))),
            }
        }
    }
}

impl<V: Ord + Sized + Default + Clone> Tree<V> {
    pub fn from_tree(other: &Self) -> Self {
        let tree = Self::new();

        if other.is_empty() {
            return tree;
        }
        
        unsafe {
            let mut stack: Vec<CloneStackEntry<V>> = Vec::with_capacity(other.height());

            stack.push(CloneStackEntry {
                new_ptr: std::ptr::null_mut(),
                ptr: (*other.sentinel.as_ptr()).right.unwrap().as_ptr(),
                state: StackState::Left,
            });
            let mut last = stack.last_mut().unwrap();
            let mut dir = false; // true is left, false is right
            let mut parent = tree.sentinel.as_ptr();

            loop {
                match last.state {
                    StackState::Left => {
                        let new_node = Box::into_raw(Box::new(Node {
                            count: (*last.ptr).count,
                            height: (*last.ptr).height,
                            left: None,
                            right: None,
                            parent: Some(NonNull::new_unchecked(parent)),
                            is_left_child: dir,
                            val: (*last.ptr).val.clone()
                        }));

                        if dir {
                            (*parent).left = Some(NonNull::new_unchecked(new_node));
                        } else {
                            (*parent).right = Some(NonNull::new_unchecked(new_node));
                        }

                        last.new_ptr = new_node;
                        last.state = StackState::Right;

                        if let Some(child) = (*last.ptr).left {
                            let entry = CloneStackEntry {
                                new_ptr: std::ptr::null_mut(),
                                ptr: child.clone().as_ptr(),
                                state: StackState::Left,
                            };

                            dir = true;
                            parent = new_node;

                            stack.push(entry);
                            last = stack.last_mut().unwrap();
                            
                        }
                    }
                    StackState::Right => {
                        last.state = StackState::Handle;

                        if let Some(child) = (*last.ptr).right {
                            let entry = CloneStackEntry {
                                new_ptr: std::ptr::null_mut(),
                                ptr: child.clone().as_ptr(),
                                state: StackState::Left,
                            };

                            dir = false;
                            parent = last.new_ptr;

                            stack.push(entry);
                            last = stack.last_mut().unwrap();
                        }
                    }
                    StackState::Handle => {
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
        };

        tree
    }
}

impl<V: Ord + Sized + Default + Clone> Clone for Tree<V> {
    fn clone(&self) -> Self {
        Self::from_tree(self)
    }
}