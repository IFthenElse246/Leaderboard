use std::ptr::NonNull;

use bincode::{de::{read::Reader, Decoder}, enc::write::Writer, Decode, Encode};

use crate::board::tree::{node::Node, tree_struct::stacks::{StackEntry, StackState}};

use super::Tree;

impl<V: Ord + Sized + Default + Clone + Encode> Encode for Tree<V> {
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        unsafe {
            if (*self.sentinel.as_ptr()).right.is_none() {
                encoder.writer().write(&[0])?;
                return  Ok(());
            } else {
                encoder.writer().write(&[1])?;
            }

            let mut stack: Vec<StackEntry<V>> = Vec::with_capacity(self.height());

            stack.push(StackEntry {
                ptr: (*self.sentinel.as_ptr()).right.unwrap().as_ptr(),
                state: StackState::Left,
            });
            let mut last = stack.last_mut().unwrap();

            loop {
                match last.state {
                    StackState::Left => {
                        Encode::encode(&(*last.ptr).val, encoder)?;

                        last.state = StackState::Right;

                        if let Some(child) = (*last.ptr).left {
                            encoder.writer().write(&[1])?;

                            let entry = StackEntry {
                                ptr: child.clone().as_ptr(),
                                state: StackState::Left,
                            };

                            stack.push(entry);
                            last = stack.last_mut().unwrap();
                        } else {
                            encoder.writer().write(&[0])?;
                        }
                    }
                    StackState::Right => {
                        last.state = StackState::Handle;

                        if let Some(child) = (*last.ptr).right {
                            encoder.writer().write(&[1])?;

                            let entry = StackEntry {
                                ptr: child.clone().as_ptr(),
                                state: StackState::Left,
                            };

                            stack.push(entry);
                            last = stack.last_mut().unwrap();
                        } else {
                            encoder.writer().write(&[0])?;
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
            };
            Ok(())
        }
    }
}

impl<V: Ord + Sized + Default + Clone + Decode<Context>, Context: bincode::de::Decoder> Decode<Context> for Tree<V> {
    fn decode<D: bincode::de::Decoder>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError>  where V: Decode<<D as Decoder>::Context> {
        unsafe {
            let mut stack: Vec<StackEntry<V>> = Vec::new();

            let tree: Tree<V> = Tree::new();
            let mut existence = [0];
            let mut val: V;

            stack.push(StackEntry {
                ptr: tree.sentinel.as_ptr(),
                state: StackState::Right,
            });
            let mut last = stack.last_mut().unwrap();

            loop {
                match last.state {
                    StackState::Left => {
                        decoder.reader().read(&mut existence)?;
                        val = bincode::Decode::decode(decoder)?;

                        last.state = StackState::Right;

                        if existence[0] == 1 {
                            let node = Node {
                                count: 0,
                                height: 0,
                                left: None,
                                right: None,
                                parent: Some(NonNull::new_unchecked(last.ptr)),
                                is_left_child: true,
                                val: val
                            };
                            let node_ptr = Box::into_raw(Box::new(node));

                            (*last.ptr).left = Some(NonNull::new_unchecked(node_ptr));

                            let entry = StackEntry {
                                ptr: node_ptr,
                                state: StackState::Left,
                            };

                            stack.push(entry);
                            last = stack.last_mut().unwrap();
                        }
                    }
                    StackState::Right => {
                        decoder.reader().read(&mut existence)?;
                        val = bincode::Decode::decode(decoder)?;

                        last.state = StackState::Handle;

                        if existence[0] == 1 {
                            let node = Node {
                                count: 0,
                                height: 0,
                                left: None,
                                right: None,
                                parent: Some(NonNull::new_unchecked(last.ptr)),
                                is_left_child: false,
                                val: val
                            };
                            let node_ptr = Box::into_raw(Box::new(node));

                            (*last.ptr).right = Some(NonNull::new_unchecked(node_ptr));

                            let entry = StackEntry {
                                ptr: node_ptr,
                                state: StackState::Left,
                            };

                            stack.push(entry);
                            last = stack.last_mut().unwrap();
                        }
                    }
                    StackState::Handle => {
                        Node::fix(last.ptr);
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

            return Ok(tree);
        }
    }
}