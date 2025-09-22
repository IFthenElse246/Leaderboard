use super::Entry;
use super::Tree;
use std::collections::HashMap;
use std::ptr::NonNull;
use std::time::{SystemTime, UNIX_EPOCH};

fn current_time() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub struct Board<K: PartialOrd + Sized + Default = u64, V: PartialOrd + Default + ?Sized = f64> {
    tree: Tree<Entry<K, V>>,
    map: HashMap<K, Entry<K, V>>,
}

impl<K: Ord + Sized + Default, V: PartialOrd + Default + ?Sized> Board<K, V> {
    pub fn new() -> Self {
        Board {
            tree: Tree::new(),
            map: HashMap::new()
        }
    }
}

unsafe impl<K: Ord + Sized + Default + Send, V: PartialOrd + Default + ?Sized + Send> Send for Board<K, V> {}
unsafe impl<K: Ord + Sized + Default + Sync, V: PartialOrd + Default + ?Sized + Sync> Sync for Board<K, V> {}