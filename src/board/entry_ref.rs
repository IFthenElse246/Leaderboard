use super::Entry;
use std::{cmp, ptr::NonNull};
use std::fmt::Display;

#[derive(PartialEq)]
pub struct EntryRef<K: Ord + Sized + Default, V: PartialOrd + ?Sized> {
    entry: Option<NonNull<Entry<V>>>,
    key: K,
}

impl<K: Ord + Sized + Default, V: PartialOrd + ?Sized> EntryRef<K, V> {
    pub fn new(key: K, entry: NonNull<Entry<V>>) -> Self {
        Self {
            entry: Some(entry),
            key: K::default()
        }
    }
}

impl<K: Ord + Sized + Default, V: PartialOrd + ?Sized> PartialOrd for EntryRef<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        unsafe {
            if self
            .entry
            .expect("Attempt to partial compare Sentinel entry")
            .as_ref()
            != other
                .entry
                .expect("Attempt to partial compare to Sentinel entry")
                .as_ref()
            {
                return self
                    .entry
                    .unwrap()
                    .as_ref()
                    .partial_cmp(other.entry.unwrap().as_ref());
            }
            return Some(self.key.cmp(&other.key));
        }
    }
}

impl<K: Ord + Sized + Default, V: PartialOrd + ?Sized> Default for EntryRef<K, V> {
    fn default() -> Self {
        Self {
            entry: None,
            key: K::default(),
        }
    }
}

impl<K: Ord + Sized + Default, V: PartialOrd + ?Sized> Ord for EntryRef<K, V> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        unsafe {
            match self
                .entry
                .expect("Attempt to compare Sentinel entry")
                .as_ref()
                .partial_cmp(
                    other
                        .entry
                        .expect("Attempt to compare to Sentinel entry")
                        .as_ref(),
                ) {
                Some(cmp::Ordering::Equal) | None => self.key.cmp(&other.key),
                Some(v) => v,
            }
        }
    }
}

impl<K: Ord + Sized + Default, V: PartialOrd + ?Sized> Eq for EntryRef<K, V> {}

impl<K: Ord + Sized + Default + Clone, V: PartialOrd + ?Sized + Clone> Clone for EntryRef<K, V> {
    fn clone(&self) -> Self {
        Self {
            entry: self.entry.clone(),
            key: self.key.clone()
        }
    }
}

unsafe impl<K: Ord + Sized + Default + Send, V: PartialOrd + ?Sized + Send> Send for EntryRef<K, V> {}
unsafe impl<K: Ord + Sized + Default + Sync, V: PartialOrd + ?Sized + Sync> Sync for EntryRef<K, V> {}