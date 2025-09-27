use super::Entry;
use super::Tree;
use std::collections::HashMap;
use std::hash::Hash;
use std::time::{SystemTime, UNIX_EPOCH};

fn current_time() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub struct Board<
    K: PartialOrd + Eq + Hash + Sized + Default + Clone = u64,
    V: PartialOrd + Default + ?Sized + Clone = f64,
> {
    tree: Tree<Entry<K, V>>,
    map: HashMap<K, Entry<K, V>>,
    size_cap: Option<usize>
}

impl<K: PartialOrd + Eq + Hash + Sized + Default + Clone, V: PartialOrd + Default + ?Sized + Clone>
    Board<K, V>
{
    pub fn get_entry(&self, id: &K) -> Option<&Entry<K, V>> {
        return self.map.get(id);
    }

    pub fn get_tree_copy(&self) -> Tree<Entry<K, V>> {
        self.tree.clone()
    }

    pub fn add_entry(&mut self, entry: Entry<K, V>) -> Result<bool, String> {
        let id = entry.key.clone();

        if self.map.contains_key(&id) {
            return Ok(false);
        }

        if let Some(cap) = self.size_cap && self.is_at_size_cap() && self.tree.index_of(&entry).0 >= cap {
            return Err("Too low rank to fall into the size cap.".to_string());
        }

        self.tree.insert(entry.clone());
        self.map.insert(id, entry);

        if self.is_past_size_cap() {
            let mut cursor = self.tree.cursor_mut();
            let entry = cursor.delete_next().unwrap();
            self.map.remove(&entry.key);
        }

        return Ok(true);
    }

    pub fn remove_entry(&mut self, id: &K) -> Option<Entry<K, V>> {
        let entry = self.map.remove(id)?;
        self.tree.remove(&entry);
        Some(entry)
    }

    pub fn set_size_cap(&mut self, new_size_cap: usize) {
        self.size_cap = Some(new_size_cap);
    }

    pub fn remove_size_cap(&mut self) {
        self.size_cap = None;
    }

    pub fn trim_after_cap(&mut self) {
        if self.size_cap.is_none() {
            return;
        }

        let cap = self.size_cap.unwrap();
        let mut cursor = self.tree.cursor_mut();
        while cap < cursor.get_tree().len() {
            let entry = cursor.delete_next().unwrap();
            self.map.remove(&entry.key);
        }
    }

    pub fn get_size_cap(&self) -> Option<usize> {
        return self.size_cap
    }

    pub fn is_at_size_cap(&self) -> bool {
        return self.size_cap.is_some() && self.size_cap.unwrap() <= self.get_size()
    }

    pub fn is_past_size_cap(&self) -> bool {
        return self.size_cap.is_some() && self.size_cap.unwrap() < self.get_size()
    }

    pub fn update_entry(&mut self, id: K, points: V) -> Result<bool,String> {
        let old_entry_opt = self.map.get(&id);
        if let None = old_entry_opt {
            let new_entry = Entry {
                key: id,
                points: points,
                timestamp: current_time(),
            };
            return match self.add_entry(new_entry) {
                Ok(_) => Ok(true),
                Err(v) => Err(v)
            };
        }
        let old_entry = old_entry_opt.unwrap();
        if old_entry.points == points {
            return Ok(true);
        }

        let new_entry = Entry {
            key: id.clone(),
            points: points,
            timestamp: current_time(),
        };

        self.tree.replace(old_entry, new_entry.clone());
        self.map.insert(id, new_entry);
        Ok(true)
    }

    pub fn get_rank(&self, id: &K) -> Option<usize> {
        let entry = self.map.get(id)?;
        return Some(self.tree.index_of(entry).0 + 1);
    }

    pub fn get_size(&self) -> usize {
        self.tree.len()
    }

    pub fn get_top(&self, count: usize) -> Vec<(usize, Entry<K, V>)> {
        let mut ret = Vec::with_capacity(count);

        let mut cursor = self.tree.cursor();

        for _i in 0..count {
            cursor.move_prev();
            if let Some(v) = cursor.get_value() {
                let entry = v.clone();
                ret.push((cursor.get_index().unwrap(), entry));
            } else {
                break;
            }
        }

        return ret;
    }

    pub fn get_bottom(&self, count: usize) -> Vec<(usize, Entry<K, V>)> {
        let mut ret = Vec::with_capacity(count);

        let mut cursor = self.tree.cursor();

        for _i in 0..count {
            cursor.move_next();
            if let Some(v) = cursor.get_value() {
                let entry = v.clone();
                ret.push((cursor.get_index().unwrap(), entry));
            } else {
                break;
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
            tree: Tree::new(),
            map: HashMap::new(),
            size_cap: None
        }
    }

    pub fn get_after(&self, id: &K, count: usize) -> Option<Vec<(usize, Entry<K, V>)>> {
        let entry = self.map.get(id)?;
        let mut ret = Vec::with_capacity(count);

        let mut cursor = self.tree.seek_val(entry).unwrap();
        for _i in 0..count {
            cursor.move_prev();
            if let Some(v) = cursor.get_value() {
                let entry = v.clone();
                ret.push((cursor.get_index().unwrap(), entry));
            } else {
                break;
            }
        }

        Some(ret)
    }

    pub fn get_before(&self, id: &K, count: usize) -> Option<Vec<(usize, Entry<K, V>)>> {
        let entry = self.map.get(id)?;
        let mut ret = Vec::with_capacity(count);

        let mut cursor = self.tree.seek_val(entry).unwrap();
        for _i in 0..count {
            cursor.move_next();
            if let Some(v) = cursor.get_value() {
                let entry = v.clone();
                ret.push((cursor.get_index().unwrap(), entry));
            } else {
                break;
            }
        }

        Some(ret)
    }

    pub fn from_tree(tree: Tree<Entry<K, V>>) -> Self {
        let mut map = HashMap::with_capacity(tree.len());

        let mut cursor = tree.cursor();
        cursor.move_next();

        while !cursor.is_at_end() {
            map.insert(
                cursor.get_value().unwrap().key.clone(),
                cursor.get_value().unwrap().clone(),
            );
            cursor.move_next();
        }

        map.shrink_to_fit();

        Self {
            tree: tree,
            map: map,
            size_cap: None
        }
    }
}

unsafe impl<
    K: PartialOrd + Eq + Hash + Sized + Default + Clone + Send,
    V: PartialOrd + Default + ?Sized + Clone + Send,
> Send for Board<K, V>
{
}
unsafe impl<
    K: PartialOrd + Eq + Hash + Sized + Default + Clone + Sync,
    V: PartialOrd + Default + ?Sized + Clone + Sync,
> Sync for Board<K, V>
{
}
