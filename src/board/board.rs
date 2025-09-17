use super::EntryRef;
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

pub struct Board<K: Ord + Sized + Default + Clone = u64, V: PartialOrd + ?Sized + Clone = f64> {
    tree: Tree<EntryRef<K, V>>,
    map: HashMap<K, NonNull<EntryRef<K, V>>>,
}

impl<K: Ord + Sized + Default + Clone, V: PartialOrd + ?Sized + Clone> Board<K, V> {
    pub fn new() -> Self {
        Board {
            tree: Tree::new(),
            map: HashMap::new()
        }
    }
}

unsafe impl<K: Ord + Sized + Default + Clone + Send, V: PartialOrd + ?Sized + Clone + Send> Send for Board<K, V> {}
unsafe impl<K: Ord + Sized + Default + Clone + Sync, V: PartialOrd + ?Sized + Clone + Sync> Sync for Board<K, V> {}