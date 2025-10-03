use std::{collections::HashMap, hash::Hash, mem, sync::{Arc, Mutex, RwLock, RwLockReadGuard}};

use bincode::Encode;

pub struct DiffMap<K, V>
where K: Eq + Hash + Clone,
    V: Clone {
    inner: Arc<Mutex<Inner<K, V>>>,
    map: Arc<RwLock<HashMap<K, V>>>,
}

struct Inner<K, V>
where K: Eq + Hash + Clone,
    V: Clone {
        num_borrows: usize,
        cleared: bool,
        diff: HashMap<K, Option<V>>
}

pub struct SnapshotBorrow<K, V>
where K: Eq + Hash + Clone,
    V: Clone {
    diff_map: DiffMap<K, V>
}

impl<K, V> DiffMap<K, V> 
where K: Eq + Hash + Clone,
    V: Clone {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                num_borrows: 0,
                cleared: false,
                diff: HashMap::new()
            })),
            map: Arc::new(RwLock::new(HashMap::new()))
        }
    }

    pub fn from_map(map: HashMap<K, V>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                num_borrows: 0,
                cleared: false,
                diff: HashMap::new()
            })),
            map: Arc::new(RwLock::new(map))
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                num_borrows: 0,
                cleared: false,
                diff: HashMap::new()
            })),
            map: Arc::new(RwLock::new(HashMap::with_capacity(capacity)))
        }
    }

    pub fn shrink_to_fit(&mut self) {
        let inner = self.inner.lock().unwrap();
        
        if inner.num_borrows == 0 {
            self.map.write().unwrap().shrink_to_fit();
        }
    }

    pub fn get<'a>(&'a self, key: &K) -> Option<V> {
        let inner = self.inner.lock().unwrap();
        
        let ret = match inner.diff.get(key) {
            None => {
                if inner.cleared {
                    None
                } else {
                    self.map.read().unwrap().get(key).cloned()
                }
            },
            Some(v) => {
                return v.as_ref().cloned();
            }
        };

        let _ = drop(inner);

        return ret;
    }

    pub fn insert(&mut self, key: K, val: V) -> Option<V> {
        let mut inner = self.inner.lock().unwrap();

        let ret = if inner.num_borrows == 0 {
            self.map.write().unwrap().insert(key, val)
        } else {
            match inner.diff.insert(key.clone(), Some(val)) {
                None => {
                    if inner.cleared {
                        None
                    } else {
                        self.map.read().unwrap().get(&key).cloned()
                    }
                },
                Some(v) => v
            }
        };

        let _ = drop(inner);

        return ret;
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let mut inner = self.inner.lock().unwrap();

        let ret = if inner.num_borrows == 0 {
            self.map.write().unwrap().remove(key)
        } else {
            if let Some(base_val) = self.map.read().unwrap().get(key) {
                match inner.diff.insert(key.clone(), None) {
                    None => {
                        if inner.cleared {
                            None
                        } else {
                            Some(base_val.clone())
                        }
                    },
                    Some(v) => v
                }
            } else {
                match inner.diff.remove(key) {
                    None => None,
                    Some(v) => v
                }
            }
        };

        let _ = drop(inner);

        return ret;
    }

    pub fn contains_key(&self, key: &K) -> bool {
        let inner = self.inner.lock().unwrap();

        let ret = match inner.diff.get(key) {
            None => {
                !inner.cleared && self.map.read().unwrap().contains_key(key)
            },
            Some(v) => {
                return v.is_some();
            }
        };

        let _ = drop(inner);

        return ret;
    }

    pub fn clear(&mut self) {
        let mut inner = self.inner.lock().unwrap();

        if inner.num_borrows == 0 {
            self.map.write().unwrap().clear();
        } else {
            inner.cleared = true;
            inner.diff.clear();
        }

        let _ = drop(inner);
    }

    pub fn snapshot_borrow<'a, 'b>(&'a self) -> SnapshotBorrow<K, V> {
        let mut inner = self.inner.lock().unwrap();
        inner.num_borrows += 1;

        let ret = SnapshotBorrow {
            diff_map: DiffMap {
                inner: self.inner.clone(),
                map: self.map.clone()
            }
        };

        let _ = drop(inner);
        return ret;
    }

    pub fn is_borrowed(&self) -> bool {
        self.inner.lock().unwrap().num_borrows > 0
    }
}

impl<'a, K, V> Drop for SnapshotBorrow<K, V>
where K: Eq + Hash + Clone,
    V: Clone {
    fn drop(&mut self) {
        let mut inner = self.diff_map.inner.lock().unwrap();
        inner.num_borrows -= 1;

        if inner.num_borrows == 0 {
            let mut map = self.diff_map.map.write().unwrap();

            if inner.cleared {
                map.clear();
            }

            let diff = mem::replace(&mut inner.diff, HashMap::new()).into_iter();
            for elem in diff {
                match elem.1 {
                    None => {
                        map.remove(&elem.0);
                    },
                    Some(v) => {
                        map.insert(elem.0, v);
                    }
                }
            }
        }
    }
}

impl<K, V> SnapshotBorrow<K, V>
where K: Eq + Hash + Clone,
    V: Clone {
    fn get_lock<'b>(&'b self) -> RwLockReadGuard<'b, HashMap<K, V>> {
        self.diff_map.map.read().unwrap()
    }
}

impl<K, V> Encode for SnapshotBorrow<K, V>
where K: Eq + Hash + Clone + Encode,
    V: Clone + Encode {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        let lock = self.get_lock();
        
        bincode::Encode::encode(&*lock, encoder)?;

        Ok(())
    }
}