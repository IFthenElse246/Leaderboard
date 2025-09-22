use std::{cmp, ptr::NonNull};

use super::Node;
use super::Tree;
use super::stacks::*;

impl<V: Ord + Sized + Default> Tree<V> {
    pub fn insert(&mut self, val: V) -> bool {
        return self.insert_node(val).is_some();
    }

    pub(super) fn insert_node(&mut self, val: V) -> Option<NonNull<Node<V>>> {
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
                return Some(node);
            }

            // left is true, right is false
            let mut dir: bool;
            let mut parent = (*self.sentinel.as_ptr()).right.unwrap().as_ptr().clone();

            loop {
                match val.cmp(&(*parent).val) {
                    cmp::Ordering::Equal => {
                        return None;
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

            return Some(new_node);
        }
    }

    pub fn replace(&mut self, old_val: &V, new_val: V) -> Option<V> {
        let mut ind: usize = 0;
        unsafe {
            let mut parent = match (*self.sentinel.as_ptr()).right {
                None => {
                    return None;
                }
                Some(ptr) => ptr.as_ptr(),
            };

            loop {
                match old_val.cmp(&(*parent).val) {
                    cmp::Ordering::Equal => {
                        ind += Node::get_right_count(parent);
                        return Some(self.replace_node(parent, ind, new_val)?.0);
                    }
                    cmp::Ordering::Greater => {
                        if (*parent).right.is_none() {
                            return None;
                        }
                        parent = (*parent).right.unwrap().as_ptr();
                    }
                    cmp::Ordering::Less => {
                        ind += 1 + Node::get_right_count(parent);
                        if (*parent).left.is_none() {
                            return None;
                        }
                        parent = (*parent).left.unwrap().as_ptr();
                    }
                };
            }
        }
    }

    pub(super) fn replace_node(
        &mut self,
        old_node: *mut Node<V>,
        old_ind: usize,
        new_val: V,
    ) -> Option<(V, NonNull<Node<V>>, usize)> {
        let index_ret = self.index_of(&new_val);
        if index_ret.1 {
            return None;
        }

        let mut new_ind = index_ret.0;
        if new_ind > old_ind {
            new_ind -= 1;
        }
        let distance = new_ind.abs_diff(old_ind);

        unsafe {
            if distance == 0 {
                let mut ret = new_val;
                std::mem::swap(&mut (*old_node).val, &mut ret);
                return Some((ret, NonNull::new_unchecked(old_node), new_ind));
            } else if distance <= self.height() / 5 {
                let mut nodes: Vec<*mut Node<V>> = Vec::with_capacity(distance + 1);
                nodes.push(old_node);

                let mut progress_node = old_node;
                if new_ind > old_ind {
                    for _ in 0..distance {
                        progress_node = Node::prev_node(progress_node);
                        nodes.push(progress_node);
                    }
                } else {
                    for _ in 0..distance {
                        progress_node = Node::next_node(progress_node);
                        nodes.push(progress_node);
                    }
                }

                let val = self.shift_nodes(&mut nodes, new_val);
                return Some((val, NonNull::new_unchecked(nodes.pop().unwrap()), new_ind));
            } else {
                self.remove_node(old_node);
                let result = self.insert_node(new_val).unwrap();
                return Some((Box::from_raw(old_node).val, result, new_ind));
            }
        }
    }

    fn shift_nodes(&mut self, nodes: &mut Vec<*mut Node<V>>, fill_val: V) -> V {
        if nodes.len() < 2 {
            panic!("Attempt to shift with 1 or fewer nodes!")
        }

        unsafe {
            let mut node1;
            let mut node2;

            for i in 0..(nodes.len() - 1) {
                node1 = nodes[i];
                node2 = nodes[i + 1];

                std::mem::swap(&mut (*node1).val, &mut (*node2).val);
            }

            let mut ret = fill_val;
            std::mem::swap(&mut (**nodes.last().unwrap()).val, &mut ret);

            return ret;
        }
    }

    pub fn remove(&mut self, val: &V) -> Option<V> {
        unsafe {
            (*self.sentinel.as_ptr()).right?;
            let mut node = (*self.sentinel.as_ptr()).right.unwrap().as_ptr();

            loop {
                match val.cmp(&(*node).val) {
                    cmp::Ordering::Equal => {
                        break;
                    }
                    cmp::Ordering::Greater => {
                        node = (*node).right?.as_ptr();
                    }
                    cmp::Ordering::Less => {
                        node = (*node).left?.as_ptr();
                    }
                };
            }

            self.remove_node(node);
            return Some((*Box::from_raw(node)).val);
        }
    }

    pub(super) fn remove_node(&mut self, node: *mut Node<V>) {
        unsafe {
            let parent: *mut Node<V> = match (*node).parent {
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
            } else {
                /* If I have two children, then find the node "before" me in the tree.
                That node will have no right child, so I can recursively delete it.
                When I'm done, I'll swap out this node with that one. */
                let mut replace_node = (*node).left.unwrap().as_ptr();
                while (*replace_node).right.is_some() {
                    replace_node = (*replace_node).right.unwrap().as_ptr();
                }
                self.remove_node(replace_node);

                let parent: *mut Node<V> = (*node).parent.unwrap().as_ptr();

                if (*node).is_left_child {
                    (*parent).left = Some(NonNull::new_unchecked(replace_node));
                } else {
                    (*parent).right = Some(NonNull::new_unchecked(replace_node));
                }

                (*replace_node).parent = (*node).parent;
                (*replace_node).is_left_child = (*node).is_left_child;

                (*replace_node).left = (*node).left;
                if let Some(child) = (*replace_node).left {
                    (*child.as_ptr()).parent = Some(NonNull::new_unchecked(replace_node));
                }

                (*replace_node).right = (*node).right;
                if let Some(child) = (*replace_node).right {
                    (*child.as_ptr()).parent = Some(NonNull::new_unchecked(replace_node));
                }

                (*replace_node).count = (*node).count;
                (*replace_node).height = (*node).height

                //self.recursive_fix_up(replace_node);
            }
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
