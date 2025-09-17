use super::Entry;
use std::{cmp, ptr::NonNull};

#[derive(PartialEq)]
pub struct EntryRef<K: Ord + Sized + Default + Clone, V: PartialOrd + ?Sized + Clone> {
    entry: Option<Entry<V>>,
    key: K,
}

impl<K: Ord + Sized + Default + Clone, V: PartialOrd + ?Sized + Clone> EntryRef<K, V> {
    pub fn new(key: K, entry: Entry<V>) -> Self {
        Self {
            entry: Some(entry),
            key: K::default()
        }
    }
}

impl<K: Ord + Sized + Default + Clone, V: PartialOrd + ?Sized + Clone> PartialOrd for EntryRef<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        if self
            .entry
            .as_ref()
            .expect("Attempt to partial compare Sentinel entry")
            != other
                .entry
                .as_ref()
                .expect("Attempt to partial compare to Sentinel entry")
        {
            return self
                .entry
                .as_ref()
                .unwrap()
                .partial_cmp(other.entry.as_ref().unwrap());
        }
        return Some(self.key.cmp(&other.key));
    }
}

impl<K: Ord + Sized + Default + Clone, V: PartialOrd + ?Sized + Clone> Default for EntryRef<K, V> {
    fn default() -> Self {
        Self {
            entry: None,
            key: K::default(),
        }
    }
}

impl<K: Ord + Sized + Default + Clone, V: PartialOrd + ?Sized + Clone> Ord for EntryRef<K, V> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self
            .entry
            .as_ref()
            .expect("Attempt to compare Sentinel entry")
            .partial_cmp(
                other
                    .entry
                    .as_ref()
                    .expect("Attempt to compare to Sentinel entry"),
            ) {
            Some(cmp::Ordering::Equal) | None => self.key.cmp(&other.key),
            Some(v) => v,
        }
    }
}

impl<K: Ord + Sized + Default + Clone, V: PartialOrd + ?Sized + Clone> Eq for EntryRef<K, V> {}

impl<K: Ord + Sized + Default + Clone, V: PartialOrd + ?Sized + Clone> Clone for EntryRef<K, V> {
    fn clone(&self) -> Self {
        Self {
            entry: self.entry.clone(),
            key: self.key.clone()
        }
    }
}

unsafe impl<K: Ord + Sized + Default + Send + Clone, V: PartialOrd + ?Sized + Send + Clone> Send for EntryRef<K, V> {}
unsafe impl<K: Ord + Sized + Default + Sync + Clone, V: PartialOrd + ?Sized + Sync + Clone> Sync for EntryRef<K, V> {}