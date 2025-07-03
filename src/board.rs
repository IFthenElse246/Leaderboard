use crate::tree::Entry;
use indexset::BTreeSet;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

fn current_time() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub struct Board<T: PartialOrd + Serialize + for<'a> Deserialize<'a> = f64> {
    tree: BTreeSet<Arc<Entry<T>>>,
    map: HashMap<u64, Arc<Entry<T>>>,
}

impl<T: PartialOrd + Serialize + for<'a> Deserialize<'a>> Board<T> {
    pub fn get_entry(&self, id: u64) -> Option<&Arc<Entry<T>>> {
        return self.map.get(&id);
    }

    pub fn get_tree_copy(&self) -> BTreeSet<Arc<Entry<T>>> {
        self.tree.clone()
    }

    pub fn add_entry(&mut self, entry: Entry<T>) -> bool {
        let id = entry.user_id;
        let arc = Arc::new(entry);
        let arc2 = arc.clone();

        if self.map.contains_key(&id) {
            return false;
        }

        self.tree.insert(arc);
        self.map.insert(id, arc2);
        return true;
    }

    pub fn remove_entry(&mut self, id: u64) -> Option<Arc<Entry<T>>> {
        let option = self.map.remove(&id);
        if let None = option {
            return None;
        }
        let entry = option.unwrap();
        self.tree.remove(&entry);
        Some(entry)
    }

    fn remove_entry_(&mut self, entry: Arc<Entry<T>>) -> bool {
        if let None = self.map.remove(&entry.user_id) {
            return false;
        }
        return self.tree.remove(&entry);
    }

    pub fn update_entry(&mut self, id: u64, points: T) -> bool {
        let old_entry_opt = self.get_entry(id);
        if let None = old_entry_opt {
            self.add_entry(Entry {
                user_id: id,
                points: points,
                timestamp: current_time(),
            });
            return false;
        }
        let old_entry = old_entry_opt.unwrap();
        let mut timestamp = old_entry.timestamp;
        if old_entry.points == points {
            return true;
        } else if old_entry.points < points {
            timestamp = current_time();
        }
        self.remove_entry_(old_entry.clone());
        self.add_entry(Entry {
            user_id: id,
            points: points,
            timestamp: timestamp,
        });
        true
    }

    pub fn get_rank(&self, id: u64) -> Option<usize> {
        let entry = self.get_entry(id)?;
        return Some(self.tree.len() - self.tree.rank(entry));
    }

    pub fn get_size(&self) -> usize {
        self.tree.len()
    }

    pub fn get_top(&self, count: usize) -> Vec<Arc<Entry<T>>> {
        let mut ret = Vec::new();

        let mut iter = self.tree.iter();
        for _i in 1..=count {
            match iter.next() {
                Some(entry) => {
                    ret.push(entry.clone());
                }
                None => {
                    break;
                }
            }
        }

        return ret;
    }

    pub fn get_bottom(&self, count: usize) -> Vec<Arc<Entry<T>>> {
        let mut ret = Vec::new();

        let mut iter = self.tree.iter().rev();
        for _i in 1..=count {
            match iter.next() {
                Some(entry) => {
                    ret.push(entry.clone());
                }
                None => {
                    break;
                }
            }
        }

        return ret;
    }

    pub fn clear(&mut self) {
        self.tree.clear();
        self.map.clear();
    }

    pub fn new() -> Self {
        Self {
            tree: BTreeSet::new(),
            map: HashMap::new(),
        }
    }

    pub fn from_file(path: &PathBuf) -> Result<Self, ciborium::de::Error<std::io::Error>> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let file = match File::open(path) {
            Ok(file) => file,
            Err(err) => {
                return Err(ciborium::de::Error::Io(err));
            }
        };
        let tree: BTreeSet<Arc<Entry<T>>> = match ciborium::from_reader(file) {
            Ok(tree) => tree,
            Err(err) => {
                return Err(err);
            }
        };

        let mut map = HashMap::new();

        for entry in tree.iter() {
            map.insert(entry.user_id, entry.clone());
        }

        Ok(Self {
            tree: tree,
            map: map,
        })
    }

    pub fn get_after(&self, id: u64, count: usize) -> Result<Vec<Arc<Entry<T>>>, String> {
        let entry = self
            .get_entry(id)
            .ok_or_else(|| format!("Id '{0}' not in leaderboard.", id))?;
        let mut ret = Vec::new();

        let mut first = true;
        for v in self
            .tree
            .range::<std::ops::RangeFrom<Arc<Entry<T>>>, Arc<Entry<T>>>(entry.clone()..)
        {
            if first {
                first = false;
                continue;
            }
            if ret.len() >= count {
                break;
            }
            ret.push(v.clone());
        }

        Ok(ret)
    }
}
