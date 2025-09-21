use std::{cmp, ptr::NonNull};

type Link<V> = Option<NonNull<Node<V>>>;

#[derive(PartialEq)]
pub struct Node<V: Ord + ?Sized> {
    pub(super) count: usize,
    pub(super) height: usize,

    pub(super) left: Link<V>,
    pub(super) right: Link<V>,
    pub(super) parent: Link<V>,
    pub(super) is_left_child: bool,

    pub val: V,
}

impl<V: Ord + ?Sized> Node<V> {
    unsafe fn rotate(node: *mut Self) {
        unsafe {
            // get the parent node and parent's parent node, meanwhile making sure this is a valid node to rotate around.
            let parent = match (*node).parent {
                None => panic!("Attempt to rotate about sentinel!"),
                Some(ptr) => ptr.as_ptr(),
            };

            let parent_parent = match (*parent).parent {
                None => panic!("Attempt to rotate about root node!"),
                Some(ptr) => ptr.as_ptr(),
            };

            let parent_parent_ref = (*parent).parent;
            let self_ref = match (*node).is_left_child {
                true => (*parent).left,
                false => (*parent).right,
            };
            let was_left_child = (*node).is_left_child;

            if (*parent).is_left_child {
                (*parent_parent).left = self_ref.clone();
                (*node).is_left_child = true;
            } else {
                (*parent_parent).right = self_ref.clone();
                (*node).is_left_child = false;
            }

            if was_left_child {
                (*parent).left = (*node).right;
                if let Some(child_ptr) = (*node).right {
                    let child = child_ptr.as_ptr();
                    (*child).parent = (*node).parent;
                    (*child).is_left_child = true;
                }

                (*node).right = (*node).parent;
                (*parent).parent = self_ref;
                (*parent).is_left_child = false;
            } else {
                (*parent).right = (*node).left;
                if let Some(child_ptr) = (*node).left {
                    let child = child_ptr.as_ptr();
                    (*child).parent = (*node).parent;
                    (*child).is_left_child = false;
                }

                (*node).left = (*node).parent;
                (*parent).parent = self_ref;
                (*parent).is_left_child = true;
            }

            (*node).parent = parent_parent_ref;

            Self::fix(parent);
            Self::fix(node);
        }
    }

    pub(super) fn is_sentinel(node: *mut Self) -> bool {
        unsafe {
            (*node).parent.is_none()
        }
    }

    pub(super) unsafe fn fix(node: *mut Self) {
        unsafe {
            Self::fix_count(node);
            Self::fix_height(node);
        }
    }

    pub(super) unsafe fn fix_height(node: *mut Self) {
        unsafe { (*node).height = 1 + Self::get_left_height(node).max(Self::get_right_height(node)) };
    }

    pub(super) unsafe fn get_left_height(node: *mut Self) -> usize {
        unsafe {
            match (*node).left {
                None => 0,
                Some(ptr) => (*ptr.as_ptr()).height.clone(),
            }
        }
    }

    pub(super) unsafe fn get_right_height(node: *mut Self) -> usize {
        unsafe {
            match (*node).right {
                None => 0,
                Some(ptr) => (*ptr.as_ptr()).height.clone(),
            }
        }
    }

    pub(super) unsafe fn get_left_count(node: *mut Self) -> usize {
        unsafe {
            match (*node).left {
                None => 0,
                Some(ptr) => (*ptr.as_ptr()).count,
            }
        }
    }

    pub(super) unsafe fn get_right_count(node: *mut Self) -> usize {
        unsafe {
            match (*node).right {
                None => 0,
                Some(ptr) => (*ptr.as_ptr()).count,
            }
        }
    }

    pub(super) unsafe fn fix_count(node: *mut Self) {
        unsafe {
            (*node).count = 1 + Self::get_left_count(node) + Self::get_right_count(node);
        }
    }

    pub(super) unsafe fn is_imbalanced(node: *mut Self) -> bool {
        unsafe { Self::get_left_height(node).abs_diff(Self::get_right_height(node)) > 1 }
    }

    pub(super) unsafe fn fix_imbalance(node: *mut Self) {
        unsafe {
            let rot_target: *mut Node<V>;
            let zag: bool;

            match Self::get_left_height(node).cmp(&Self::get_right_height(node)) {
                cmp::Ordering::Equal => panic!("Heights are equal for imbalance"),
                cmp::Ordering::Greater => {
                    let left = (*node).left.unwrap().as_ptr();
                    match Self::get_left_height(left).cmp(&Self::get_right_height(left)) {
                        cmp::Ordering::Equal | cmp::Ordering::Greater => {
                            zag = false;
                            rot_target = left;
                        }
                        cmp::Ordering::Less => {
                            zag = true;
                            rot_target = (*left).right.unwrap().as_ptr();
                        }
                    }
                }
                cmp::Ordering::Less => {
                    let right = (*node).right.unwrap().as_ptr();
                    match Self::get_right_height(right).cmp(&Self::get_left_height(right)) {
                        cmp::Ordering::Equal | cmp::Ordering::Greater => {
                            zag = false;
                            rot_target = right;
                        }
                        cmp::Ordering::Less => {
                            zag = true;
                            rot_target = (*right).left.unwrap().as_ptr();
                        }
                    }
                }
            };

            Self::rotate(rot_target);
            if zag {
                Self::rotate(rot_target);
            }
        }
    }

    pub(super) unsafe fn next_node(node: *mut Self) -> *mut Self {
        unsafe {
            if let Some(right) =  (*node).right {
                let mut next = right.as_ptr();
                while let Some(left) = (*next).left {
                    next = left.as_ptr();
                }
                return next;
            } else {
                let mut next = match (*node).parent {
                    Some(v) => v.as_ptr(),
                    None => {return node;}
                };
                let mut is_left = (*node).is_left_child;
                while !is_left {
                    is_left = (*next).is_left_child;
                    next = match (*next).parent {
                        None => {return next;},
                        Some(v) => v.as_ptr()
                    };
                };
                return next;
            }
        }
    }

    pub(super) unsafe fn prev_node(node: *mut Self) -> *mut Self {
        unsafe {
            if (*node).parent.is_none() {
                let mut prev = match (*node).right {
                    None => {return node;},
                    Some(v) => v.as_ptr()
                };
                while let Some(right) = (*prev).right {
                    prev = right.as_ptr();
                }
                return prev;
            }

            if let Some(left) =  (*node).left {
                let mut prev: *mut Node<V> = left.as_ptr();
                while let Some(right) = (*prev).right {
                    prev = right.as_ptr();
                }
                return prev;
            } else {
                let mut prev = (*node).parent.unwrap().as_ptr();
                let mut is_left = (*node).is_left_child;
                while is_left {
                    is_left = (*prev).is_left_child;
                    prev = match (*prev).parent {
                        None => {return prev;},
                        Some(v) => v.as_ptr()
                    };
                };
                return prev;
            }
        }
    }
}

impl<V: Ord + ?Sized> PartialOrd for Node<V> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        return self.val.partial_cmp(&other.val);
    }
}

impl<V: Ord + ?Sized> Ord for Node<V> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        return self.val.cmp(&other.val);
    }
}

impl<V: Ord + ?Sized> Eq for Node<V> {}

unsafe impl<V: Send + Ord + ?Sized> Send for Node<V> {}
unsafe impl<V: Sync + Ord + ?Sized> Sync for Node<V> {}