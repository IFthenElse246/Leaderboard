use std::{cmp::Ordering, mem};

use crate::tree_node::Node;

#[derive(PartialEq, Clone)]
pub struct Entry<T>
where
    T: PartialOrd + ?Sized,
{
    pub user_id: u64,
    pub timestamp: u128,
    pub points: T,
}

impl<T: PartialOrd> PartialOrd for Entry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.points != other.points {
            return self.points.partial_cmp(&other.points);
        }
        if other.timestamp != self.timestamp {
            return other.timestamp.partial_cmp(&self.timestamp);
        }
        return other.user_id.partial_cmp(&self.user_id);
    }
}

impl<T: PartialOrd> Ord for Entry<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.points.partial_cmp(&other.points) {
            None | Some(std::cmp::Ordering::Equal) => match other.timestamp.cmp(&self.timestamp) {
                std::cmp::Ordering::Equal => other.user_id.cmp(&self.user_id),
                x => x,
            },
            Some(x) => x,
        }
    }
}

impl<T: PartialOrd> Eq for Entry<T> {}

pub struct Tree {
    sentinel: Node<Entry<f64>>,
}

impl Tree {
    fn insert_step(new_node: Node<Entry<f64>>, parent: &mut Node<Entry<f64>>) -> bool {
        let right;
        match new_node.compare(parent) {
            Ordering::Greater => {
                right = true;
            }
            Ordering::Less => {
                right = false;
            }
            Ordering::Equal => {
                return false;
            }
        };

        let n = if right {
            &mut parent.right
        } else {
            &mut parent.left
        };
        match n {
            Some(new_parent) => {
                if !Self::insert_step(new_node, new_parent) {
                    return false;
                }

                if new_parent.is_imbalanced() {
                    parent.fix_child_imbalance(right);
                } else {
                    new_parent.fix_count();
                    new_parent.fix_height();
                }
            }
            None => {
                mem::replace(n, Some(Box::new(new_node)));
            }
        };
        return true;
    }

    pub fn insert(&mut self, entry: Entry<f64>) -> bool {
        let parent = &mut self.sentinel;
        let n = &mut parent.right;

        let new_node = Node {
            left: None,
            right: None,
            entry: entry,
            count: 1,
            height: 1,
        };

        match n {
            None => {
                mem::replace(n, Some(Box::new(new_node)));
            }
            Some(root) => {
                if !Self::insert_step(new_node, root) {
                    return false;
                }
                if root.is_imbalanced() {
                    parent.fix_child_imbalance(true);
                } else {
                    root.fix_count();
                    root.fix_height();
                }
            }
        };

        return true;
    }

    fn remove_second_step(
        parent: &mut Option<Box<Node<Entry<f64>>>>,
        right: bool,
    ) -> Box<Node<Entry<f64>>> {
        let n = if right {
            &mut parent.as_mut().unwrap().right
        } else {
            &mut parent.as_mut().unwrap().left
        };
        match n {
            Some(_) => {
                let ret = Self::remove_second_step(n, right);
                if n.as_ref().unwrap().is_imbalanced() {
                    parent.as_mut().unwrap().fix_child_imbalance(right);
                } else {
                    n.as_mut().unwrap().fix_count();
                    n.as_mut().unwrap().fix_height();
                }
                ret
            }
            None => {
                let new_child = mem::replace(
                    if right {
                        &mut parent.as_mut().unwrap().left
                    } else {
                        &mut parent.as_mut().unwrap().right
                    },
                    None,
                );
                mem::replace(parent, new_child).unwrap()
            }
        }
    }

    fn remove_step(target: &Entry<f64>, parent: &mut Node<Entry<f64>>, right: bool) -> bool {
        let n = if right {
            &mut parent.right
        } else {
            &mut parent.left
        };
        match n {
            Some(new_parent) => {
                match new_parent.compare_entry(target) {
                    Ordering::Equal => {
                        if new_parent.left.is_none() {
                            let v = mem::replace(&mut new_parent.right, None);
                            mem::replace(n, v);
                            return true;
                        } else if new_parent.right.is_none() {
                            let v = mem::replace(&mut new_parent.left, None);
                            mem::replace(n, v);
                            return true;
                        } else {
                            let start = if right {
                                &mut new_parent.left
                            } else {
                                &mut new_parent.right
                            };
                            let mut new_node = *Self::remove_second_step(start, right);
                            mem::replace(
                                &mut new_node.left,
                                mem::replace(&mut new_parent.left, None),
                            );
                            mem::replace(
                                &mut new_node.right,
                                mem::replace(&mut new_parent.right, None),
                            );
                            mem::replace(n, Some(Box::new(new_node)));
                        }
                    }
                    v => {
                        if !Self::remove_step(
                            target,
                            new_parent,
                            match v {
                                Ordering::Greater => false,
                                _ => true,
                            },
                        ) {
                            return false;
                        }
                    }
                };
            }
            None => {
                return false;
            }
        };
        if n.as_ref().unwrap().is_imbalanced() {
            parent.fix_child_imbalance(right);
        } else {
            n.as_mut().unwrap().fix_count();
            n.as_mut().unwrap().fix_height();
        }
        return true;
    }

    pub fn remove(&mut self, entry: &Entry<f64>) -> bool {
        Self::remove_step(entry, &mut self.sentinel, true)
    }

    pub fn height(&self) -> usize {
        match &self.sentinel.right {
            Some(e) => e.height,
            None => 0,
        }
    }

    pub fn size(&self) -> usize {
        match &self.sentinel.right {
            Some(e) => e.count,
            None => 0,
        }
    }

    pub fn empty(&self) -> bool {
        self.sentinel.right.is_none()
    }

    fn display_step(&self, ret: &mut String, node: &Option<Box<Node<Entry<f64>>>>, depth: usize) {
        if node.is_none() {
            return;
        }
        self.display_step(ret, &node.as_ref().unwrap().left, depth + 1);
        ret.push_str(
            format!(
                "{}{}: {} (height: {}, count: {})\n",
                "   ".repeat(depth),
                node.as_ref().unwrap().entry.user_id,
                node.as_ref().unwrap().entry.points,
                node.as_ref().unwrap().height,
                node.as_ref().unwrap().count
            )
            .as_str(),
        );
        self.display_step(ret, &node.as_ref().unwrap().right, depth + 1);
    }

    pub fn display(&self) -> String {
        let mut ret = String::new();

        self.display_step(&mut ret, &self.sentinel.right, 0);

        ret
    }

    pub fn get_rank(&self, n: &Entry<f64>) -> usize {
        let mut node = &self.sentinel.right;
        let mut rank: usize = 1;

        while node.is_some() {
            match node.as_ref().unwrap().compare_entry(n) {
                Ordering::Equal => {
                    return rank;
                }
                Ordering::Greater => {
                    rank += 1;
                    rank += match &node.as_ref().unwrap().right {
                        Some(v) => v.count,
                        None => 0,
                    };
                    node = &node.as_ref().unwrap().left;
                }
                Ordering::Less => {
                    node = &node.as_ref().unwrap().right;
                }
            }
        }

        return rank;
    }

    fn get_top_recursive(&self, node: &Option<Box<Node<Entry<f64>>>>, count: &usize, result: &mut Vec<Entry<f64>>) {
        if node.is_none() {
            return;
        }
        self.get_top_recursive(&node.as_ref().unwrap().right, count, result);
        if result.len() >= *count {
            return;
        }
        result.push(node.as_ref().unwrap().entry.clone());
        if result.len() < *count {
            self.get_top_recursive(&node.as_ref().unwrap().left, count, result);
        }
    }

    pub fn get_top(&self, count: usize) -> Vec<Entry<f64>> {
        let mut res = Vec::new();
        self.get_top_recursive(&self.sentinel.right, &count, &mut res);
        res
    }

    fn get_bottom_recursive(&self, node: &Option<Box<Node<Entry<f64>>>>, count: &usize, result: &mut Vec<Entry<f64>>) {
        if node.is_none() {
            return;
        }
        self.get_bottom_recursive(&node.as_ref().unwrap().left, count, result);
        if result.len() >= *count {
            return;
        }
        result.push(node.as_ref().unwrap().entry.clone());
        if result.len() < *count {
            self.get_bottom_recursive(&node.as_ref().unwrap().right, count, result);
        }
    }

    pub fn get_bottom(&self, count: usize) -> Vec<Entry<f64>> {
        let mut res = Vec::new();
        self.get_bottom_recursive(&self.sentinel.right, &count, &mut res);
        res
    }

    fn get_after_recursive(&self, node: &Option<Box<Node<Entry<f64>>>>, target: &Entry<f64>, count: &usize, result: &mut Vec<Entry<f64>>) {
        if node.is_none() {
            return;
        }
        match &node.as_ref().unwrap().compare_entry(target) {
            Ordering::Less => { 
                self.get_after_recursive(&node.as_ref().unwrap().right, target, count, result);
                if result.len() >= *count {
                    return;
                }
                result.push(node.as_ref().unwrap().entry.clone());
            },
            _ => {}
        }
        
        if result.len() < *count {
            self.get_after_recursive(&node.as_ref().unwrap().left, target, count, result);
        }
    }

    pub fn get_after(&self, entry: Entry<f64>, count: usize) -> Vec<Entry<f64>> {
        let mut res = Vec::new();
        self.get_after_recursive(&self.sentinel.right, &entry, &count, &mut res);
        res
    }

    fn get_before_recursive(&self, node: &Option<Box<Node<Entry<f64>>>>, target: &Entry<f64>, count: &usize, result: &mut Vec<Entry<f64>>) {
        if node.is_none() {
            return;
        }
        match &node.as_ref().unwrap().compare_entry(target) {
            Ordering::Greater => { 
                self.get_before_recursive(&node.as_ref().unwrap().left, target, count, result);
                if result.len() >= *count {
                    return;
                }
                result.push(node.as_ref().unwrap().entry.clone());
            },
            _ => {}
        }
        
        if result.len() < *count {
            self.get_before_recursive(&node.as_ref().unwrap().right, target, count, result);
        }
    }

    pub fn get_before(&self, entry: Entry<f64>, count: usize) -> Vec<Entry<f64>> {
        let mut res = Vec::new();
        self.get_before_recursive(&self.sentinel.right, &entry, &count, &mut res);
        res
    }
}
