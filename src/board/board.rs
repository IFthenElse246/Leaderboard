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

pub struct Board<K: Ord + Sized + Default = u64, V: PartialOrd + ?Sized = f64> {
    tree: Tree<EntryRef<K, V>>,
    map: HashMap<K, NonNull<EntryRef<K, V>>>,
}

impl<K: Ord + Sized + Default, V: PartialOrd + ?Sized> Board<K, V> {
    pub fn new() -> Self {
        Board {
            tree: Tree::new(),
            map: HashMap::new()
        }
    }
}

unsafe impl<K: Ord + Sized + Default + Send, V: PartialOrd + ?Sized + Send> Send for Board<K, V> {}
unsafe impl<K: Ord + Sized + Default + Sync, V: PartialOrd + ?Sized + Sync> Sync for Board<K, V> {}