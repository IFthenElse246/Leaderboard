use std::cmp;
use std::fmt::Display;

use super::Node;
use super::Tree;
use super::stacks::*;

impl<V: Ord + Sized + Default + Clone> Tree<V> {
    pub fn validate(&self) {
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
            let mut dir = false; // left is true, right is false

            loop {
                match last.state {
                    StackState::Left => {
                        if dir != (*last.ptr).is_left_child {
                            panic!("Is left child does not match if it is a left child!");
                        }
                        if let Some(left) = (*last.ptr).left {
                            if (*left.as_ptr()).parent.unwrap().as_ptr() != last.ptr {
                                panic!("Parent does not agree with left!");
                            }
                        }
                        if let Some(right) = (*last.ptr).right {
                            if (*right.as_ptr()).parent.unwrap().as_ptr() != last.ptr {
                                panic!("Parent does not agree with right!");
                            }
                        }
                        // if Node::is_imbalanced(last.ptr) {
                        //     panic!("Tree is imbalanced!");
                        // }

                        last.state = StackState::Right;
                        if let Some(child) = (*last.ptr).left {
                            match (*child.as_ptr()).cmp(&*last.ptr) {
                                cmp::Ordering::Greater => {
                                    panic!("Incorrect ordering! Node is left whilst being greater.")
                                }
                                cmp::Ordering::Equal => panic!("Multiple equal nodes in tree!"),
                                cmp::Ordering::Less => {}
                            };
                            let entry = StackEntry {
                                ptr: child.clone().as_ptr(),
                                state: StackState::Left,
                            };
                            stack.push(entry);
                            last = stack.last_mut().unwrap();
                            dir = true;
                        }
                    }
                    StackState::Right => {
                        last.state = StackState::Handle;
                        if let Some(child) = (*last.ptr).right {
                            match (*child.as_ptr()).cmp(&*last.ptr) {
                                cmp::Ordering::Less => {
                                    panic!("Incorrect ordering! Node is right whilst being lesser.")
                                }
                                cmp::Ordering::Equal => panic!("Multiple equal nodes in tree!"),
                                cmp::Ordering::Greater => {}
                            };
                            let entry = StackEntry {
                                ptr: child.clone().as_ptr(),
                                state: StackState::Left,
                            };
                            stack.push(entry);
                            last = stack.last_mut().unwrap();
                            dir = false;
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
        }
    }
}

impl<V: Ord + Sized + Default + Clone + Display> Tree<V> {
    pub fn print_pretty(&self) {
        println!("Height: {}, Size: {}", self.height(), self.len());
        let full_height = self.height();
        let mut cursor = self.cursor();
        cursor.move_prev();
        while !cursor.is_at_end() {
            print!("{}:\t", cursor.get_height().unwrap());
            for i in 0..(full_height-cursor.get_height().unwrap()) {
                print!("\t");
            }
            print!("{}\n", *cursor.get_value().unwrap());
            cursor.move_prev();
        }
    }
}