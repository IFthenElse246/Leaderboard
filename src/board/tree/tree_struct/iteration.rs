use std::{ptr::NonNull};

use crate::board::tree::{node::Node};

use super::Tree;

pub struct CursorMut<'a, V: Ord + Sized + Default + Clone> {
    tree: &'a mut Tree<V>,
    node: *mut Node<V>,
    index: Option<usize>,
    val: Option<&'a V>
}

impl<'a, V: Ord + Sized + Default + Clone> CursorMut<'a, V> {
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
}

// TODO: MAKE NON MUT VERSION
// TODO: ADD DELETE FOR MUT CURSOR
// TODO: ADD SEEK VALUE

impl<V: Ord + Sized + Default + Clone> Tree<V> {
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

    
}