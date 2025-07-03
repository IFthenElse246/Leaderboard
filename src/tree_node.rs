use std::{cmp, mem};

pub struct Node<T: Ord> {
    pub left: Option<Box<Node<T>>>,
    pub right: Option<Box<Node<T>>>,
    pub entry: T,
    pub count: usize,
    pub height: usize,
}

impl<T: Ord> Node<T> {
    pub fn rotate_child_child(&mut self, right_first: bool, right_second: bool) {
        let child_ref = if right_first {
            &mut self.right
        } else {
            &mut self.left
        };
        let mut child = mem::replace(child_ref, None).expect("Invalid path for rotate");
        let child_child_ref = if right_second {
            &mut child.right
        } else {
            &mut child.left
        };
        let mut child_child = mem::replace(child_child_ref, None)
            .expect("Cannot rotate about node that does not exist.");
        let extra_child_ref = if right_second {
            &mut child_child.left
        } else {
            &mut child_child.right
        };
        let extra_child = mem::replace(extra_child_ref, None);

        mem::replace(child_child_ref, extra_child);

        child.fix_count();
        child.fix_height();

        mem::replace(extra_child_ref, Some(child));

        child_child.fix_count();
        child_child.fix_height();

        mem::replace(child_ref, Some(child_child));
    }

    pub fn fix_height(&mut self) {
        self.height = std::cmp::max(self.get_left_height(), self.get_right_height()) + 1;
    }

    pub fn fix_count(&mut self) {
        self.count = self.get_left_count() + self.get_right_count() + 1;
    }

    pub fn get_left_height(&self) -> usize {
        match &self.left {
            Some(n) => n.height,
            None => 0,
        }
    }

    pub fn get_right_height(&self) -> usize {
        match &self.right {
            Some(n) => n.height,
            None => 0,
        }
    }

    pub fn get_left_count(&self) -> usize {
        match &self.left {
            Some(n) => n.count,
            None => 0,
        }
    }

    pub fn get_right_count(&self) -> usize {
        match &self.right {
            Some(n) => n.count,
            None => 0,
        }
    }

    pub fn compare(&self, other: &Node<T>) -> std::cmp::Ordering {
        self.entry.cmp(&other.entry)
    }

    pub fn compare_entry(&self, other: &T) -> std::cmp::Ordering {
        self.entry.cmp(other)
    }

    pub fn is_imbalanced(&self) -> bool {
        self.get_left_height().abs_diff(self.get_right_height()) > 1
    }

    pub fn fix_child_imbalance(&mut self, right: bool) {
        let mut binding = if right {
            self.right.as_mut()
        } else {
            self.left.as_mut()
        };
        let child = binding
            .as_mut()
            .expect("Cannot fix imbalance on node that does not exist.");

        match child.get_left_height().cmp(&child.get_right_height()) {
            cmp::Ordering::Greater => {
                match child
                    .left
                    .as_mut()
                    .unwrap()
                    .get_left_height()
                    .cmp(&child.left.as_mut().unwrap().get_right_height())
                {
                    cmp::Ordering::Greater => {
                        self.rotate_child_child(right, false);
                    }
                    cmp::Ordering::Less => {
                        child.rotate_child_child(false, true);
                        self.rotate_child_child(right, false);
                    }
                    cmp::Ordering::Equal => {
                        panic!("Attempt to fix imbalance when balanced.");
                    }
                };
            }
            cmp::Ordering::Less => {
                match child
                    .right
                    .as_mut()
                    .unwrap()
                    .get_left_height()
                    .cmp(&child.right.as_mut().unwrap().get_right_height())
                {
                    cmp::Ordering::Greater => {
                        child.rotate_child_child(true, false);
                        self.rotate_child_child(right, true);
                    }
                    cmp::Ordering::Less => {
                        self.rotate_child_child(right, true);
                    }
                    cmp::Ordering::Equal => {
                        panic!("Attempt to fix imbalance when balanced.");
                    }
                };
            }
            cmp::Ordering::Equal => {
                panic!("Attempt to fix imbalance when balanced.");
            }
        };
    }
}
