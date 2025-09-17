use std::marker::PhantomData;

use crate::board::tree::{node::Node, tree_struct::stacks::StackEntry};

use super::Tree;

pub struct Iter<'a, V: Ord + Sized + Default + Clone> {
    node: *mut Node<V>,
    _boo: PhantomData<&'a V>
}

impl<'a, V: Ord + Sized + Default + Clone> Iterator for Iter<'a, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {

    }
}

impl<V: Ord + Sized + Default + Clone> Tree<V> {
    pub fn iter<'a>(&'a self) -> Iter<'a, V> {
        
    }
}