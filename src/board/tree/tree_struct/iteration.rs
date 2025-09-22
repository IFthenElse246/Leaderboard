use std::cmp;

use crate::board::tree::{node::Node};

use super::Tree;

#[derive(Clone)]
pub struct Cursor<'a, V: Ord + Sized + Default + Clone > {
    tree: &'a Tree<V>,
    node: *mut Node<V>,
    index: Option<usize>,
    val: Option<&'a V>
}

macro_rules! cursor_impl {
    ($cursor:ident) => {
        impl<'a, V: Ord + Sized + Default + Clone > $cursor<'a, V> {
            // advance to the next highest person on the leaderboard. If pointing at sentinel, will point to lowest person in the leaderboard. Decreases index/rank.
            pub fn move_next<'b>(&'b mut self) -> Option<&'b V> {
                unsafe {
                    self.node = Node::next_node(self.node);
                    if (*self.node).parent.is_none() {
                        self.val = None;
                        self.index = None;
                        return None;
                    } else {
                        self.index = match self.index {
                            Some(v) => Some(v - 1),
                            None => match self.val {
                                Some(_v) => None,
                                None => Some(self.tree.len()-1)
                            }
                        };
                        self.val = Some(&(*self.node).val);
                        return self.val;
                    }
                }
            }

            // advance to next lowest person on the leaderboard. If pointing at sentinel, will point to highest person in the leaderboard. Increases index/rank.
            pub fn move_prev<'b>(&'b mut self) -> Option<&'b V> {
                unsafe {
                    self.node = Node::prev_node(self.node);
                    if (*self.node).parent.is_none() {
                        self.val = None;
                        self.index = None;
                        return None;
                    } else {
                        self.index = match self.index {
                            Some(v) => Some(v + 1),
                            None => match self.val {
                                Some(_v) => None,
                                None => Some(0)
                            }
                        };
                        self.val = Some(&(*self.node).val);
                        return self.val;
                    }
                }
            }

            pub fn move_right<'b>(&'b mut self) -> Option<&'b V> {
                unsafe {
                    self.node = match (*self.node).right {
                        None => self.tree.sentinel.as_ptr(),
                        Some(v) => v.as_ptr()
                    };
                    if (*self.node).parent.is_none() {
                        self.val = None;
                        self.index = None;
                        return None;
                    } else {
                        self.index = match self.index {
                            Some(v) => Some(v - 1 - Node::get_left_count(self.node)),
                            None => match self.val {
                                Some(_v) => None,
                                None => Some(Node::get_right_count(self.node))
                            }
                        };
                        self.val = Some(&(*self.node).val);
                        return self.val;
                    }
                }
            }

            pub fn move_left<'b>(&'b mut self) -> Option<&'b V> {
                unsafe {
                    self.node = match (*self.node).left {
                        None => self.tree.sentinel.as_ptr(),
                        Some(v) => v.as_ptr()
                    };
                    if (*self.node).parent.is_none() {
                        self.val = None;
                        self.index = None;
                        return None;
                    } else {
                        self.index = self.index.map(|v| v + 1 + Node::get_right_count(self.node));
                        self.val = Some(&(*self.node).val);
                        return self.val;
                    }
                }
            }

            pub fn move_parent<'b>(&'b mut self) -> Option<&'b V> {
                unsafe {
                    let prev_node = self.node;
                    self.node = match (*self.node).parent {
                        None => self.node,
                        Some(v) => v.as_ptr()
                    };
                    if (*self.node).parent.is_none() {
                        self.val = None;
                        self.index = None;
                        return None;
                    } else {
                        self.index = if (*prev_node).is_left_child {
                            self.index.map(|v| v - 1 - Node::get_right_count(prev_node))
                        } else {
                            self.index.map(|v| v + 1 + Node::get_left_count(prev_node))
                        };
                        self.val = Some(&(*self.node).val);
                        return self.val;
                    }
                }
            }

            pub fn get_index(&mut self) -> Option<usize> {
                match self.index {
                    Some(v) => Some(v),
                    None => match self.val {
                        None => None,
                        Some(v) => {
                            self.index = Some(self.tree.index_of(v).0);
                            self.index
                        }
                    }
                }
            }

            pub fn get_value<'b>(&'b self) -> Option<&'b V> {
                return self.val;
            }

            pub fn is_at_end(&self) -> bool {
                self.val.is_none()
            }

            pub fn has_left(&self) -> bool {
                unsafe {(*self.node).left.is_some()}
            }

            pub fn has_right(&self) -> bool {
                unsafe {(*self.node).right.is_some()}
            }

            pub fn is_root(&self) -> bool {
                unsafe {
                    let parent = (*self.node).parent;
                    parent.is_some() && (*parent.unwrap().as_ptr()).parent.is_none()
                }
            }

            pub fn get_height(&self) -> Option<usize> {
                unsafe {
                    if self.is_at_end() {
                        return None;
                    }
                    return Some((*self.node).height);
                }
            }

            pub fn get_tree<'b>(&'b self) -> &'b Tree<V> {
                return self.tree;
            } 
        }
    };
}

pub struct CursorMut<'a, V: Ord + Sized + Default + Clone > {
    tree: &'a mut Tree<V>,
    node: *mut Node<V>,
    index: Option<usize>,
    val: Option<&'a V>
}

cursor_impl!{Cursor}
cursor_impl!{CursorMut}

impl<'a, V: Ord + Sized + Default + Clone > CursorMut<'a, V> {
    pub fn delete_next(&mut self) -> Option<V> {
        unsafe {
            let target = Node::next_node(self.node);
            (*target).parent?;
            self.tree.remove_node(target);
            return Some((*Box::from_raw(target)).val);
        }
    }

    pub fn delete_prev(&mut self) -> Option<V> {
        unsafe {
            let target = Node::prev_node(self.node);
            (*target).parent?;
            self.tree.remove_node(target);
            return Some((*Box::from_raw(target)).val);
        }
    }

    pub fn replace(&mut self, val: V) -> Option<V> {
        let ind = self.get_index()?;
        let result = self.tree.replace_node(self.node, ind, val)?;
        self.index = Some(result.2);
        self.node = result.1.as_ptr();
        return Some(result.0);
    }
}

impl<V: Ord + Sized + Default + Clone > Tree<V> {
    pub fn cursor<'a>(&'a self) -> Cursor<'a, V> {
        let sentinel = self.sentinel.as_ptr();
        Cursor {
            tree: self,
            node: sentinel,
            index: None,
            val: None
        }
    }

    pub fn seek_index<'a>(&'a self, index: usize) -> Option<Cursor<'a, V>> {
        unsafe {
            let node = self.node_at_index(index)?;
            Some(Cursor {
                tree: self,
                node: node,
                index: Some(index),
                val: Some(&(*node).val)
            })
        }
    }

    pub fn seek_val<'a>(&'a self, val: &V) -> Option<Cursor<'a, V>> {
        unsafe {
            let mut node = (*self.sentinel.as_ptr()).right?.as_ptr();
            let mut index = 0;
            loop {
                match (*node).val.cmp(val) {
                    cmp::Ordering::Less => {
                        node = (*node).right?.as_ptr();
                    },
                    cmp::Ordering::Equal => {
                        index += Node::get_right_count(node);
                        break;
                    },
                    cmp::Ordering::Greater => {
                        index += 1 + Node::get_right_count(node);
                        node = (*node).left?.as_ptr();
                    }
                };
            };
            Some(Cursor {
                tree: self,
                node: node,
                index: Some(index),
                val: Some(&(*node).val)
            })
        }
    }

    pub fn cursor_mut<'a>(&'a mut self) -> CursorMut<'a, V> {
        let sentinel = self.sentinel.as_ptr();
        CursorMut {
            tree: self,
            node: sentinel,
            index: None,
            val: None
        }
    }

    pub fn seek_index_mut<'a>(&'a mut self, index: usize) -> Option<CursorMut<'a, V>> {
        unsafe {
            let node = self.node_at_index(index)?;
            Some(CursorMut {
                tree: self,
                node: node,
                index: Some(index),
                val: Some(&(*node).val)
            })
        }
    }

    pub fn seek_val_mut<'a>(&'a mut self, val: &V) -> Option<CursorMut<'a, V>> {
        unsafe {
            let mut node = (*self.sentinel.as_ptr()).right?.as_ptr();
            let mut index = 0;
            loop {
                match (*node).val.cmp(val) {
                    cmp::Ordering::Less => {
                        node = (*node).right?.as_ptr();
                    },
                    cmp::Ordering::Equal => {
                        index += Node::get_right_count(node);
                        break;
                    },
                    cmp::Ordering::Greater => {
                        index += 1 + Node::get_right_count(node);
                        node = (*node).left?.as_ptr();
                    }
                };
            };
            Some(CursorMut {
                tree: self,
                node: node,
                index: Some(index),
                val: Some(&(*node).val)
            })
        }
    }
}