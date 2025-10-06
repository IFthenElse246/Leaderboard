use bincode::Decode;
use bincode::Encode;
use bincode::de::Decoder;
use rocket::tokio::time::Instant;

use super::Entry;
use super::Tree;
use super::diff_map::{DiffMap, SnapshotBorrow};
use std::collections::HashMap;
use std::hash::Hash;
use std::time::{SystemTime, UNIX_EPOCH};

fn current_time() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}

pub struct Cache<K, V>
where
    K: PartialOrd + Eq + Hash + Sized + Default + Clone,
    V: PartialOrd + Default + ?Sized + Clone,
{
    top_cache: Option<Vec<(usize, Entry<K, V>)>>,
    bottom_cache: Option<Vec<(usize, Entry<K, V>)>>,
    top_cache_time: Instant,
    bottom_cache_time: Instant,
    top_requested: bool,
    bottom_requested: bool,
}

pub struct Board<
    K: PartialOrd + Eq + Hash + Sized + Default + Clone = u64,
    V: PartialOrd + Default + ?Sized + Clone = f64,
> {
    tree: Tree<Entry<K, V>>,
    map: DiffMap<K, Entry<K, V>>,
    size_cap: Option<usize>,
    cache: Cache<K, V>,
}

impl<K: PartialOrd + Eq + Hash + Sized + Default + Clone, V: PartialOrd + Default + ?Sized + Clone>
    Board<K, V>
{
    pub fn get_entry(&self, id: &K) -> Option<Entry<K, V>> {
        return self.map.get(id);
    }

    pub fn get_entry_and_rank(&self, id: &K) -> Option<(usize, Entry<K, V>)> {
        let entry = self.map.get(id)?;
        let rank = self.tree.index_of(&entry).0 + 1;
        Some((rank, entry))
    }

    pub fn get_tree_copy(&self) -> Tree<Entry<K, V>> {
        self.tree.clone()
    }

    pub fn get_map_snapshot(&self) -> SnapshotBorrow<K, Entry<K, V>> {
        self.map.snapshot_borrow()
    }

    pub fn is_map_snapshotted(&self) -> bool {
        self.map.is_borrowed()
    }

    pub fn add_entry(&mut self, entry: Entry<K, V>) -> Result<bool, String> {
        let id = entry.key.clone();

        if self.map.contains_key(&id) {
            return Ok(false);
        }

        if let Some(cap) = self.size_cap
            && self.is_at_size_cap()
            && self.tree.index_of(&entry).0 >= cap
        {
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

    pub fn get_ids(&self) -> Vec<K> {
        let mut ret = Vec::with_capacity(self.get_size());

        let mut cursor = self.tree.cursor();

        loop {
            let val = cursor.move_next();
            match val {
                Some(v) => {
                    ret.push(v.key.clone());
                }
                None => {
                    break;
                }
            }
        }

        return ret;
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
        return self.size_cap;
    }

    pub fn is_at_size_cap(&self) -> bool {
        return self.size_cap.is_some() && self.size_cap.unwrap() <= self.get_size();
    }

    pub fn is_past_size_cap(&self) -> bool {
        return self.size_cap.is_some() && self.size_cap.unwrap() < self.get_size();
    }

    pub fn update_entry(&mut self, id: K, points: V) -> Result<bool, String> {
        let old_entry_opt = self.map.get(&id);
        if let None = old_entry_opt {
            let new_entry = Entry {
                key: id,
                points: points,
                timestamp: current_time(),
            };
            return match self.add_entry(new_entry) {
                Ok(_) => Ok(true),
                Err(v) => Err(v),
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

        self.tree.replace(&old_entry, new_entry.clone());
        self.map.insert(id, new_entry);
        Ok(true)
    }

    pub fn get_rank(&self, id: &K) -> Option<usize> {
        let entry = self.map.get(id)?;
        return Some(self.tree.index_of(&entry).0 + 1);
    }

    pub fn at_rank(&self, rank: usize) -> Option<Entry<K, V>> {
        return self.tree.at_index(rank - 1).map(|v| v.clone());
    }

    pub fn get_size(&self) -> usize {
        self.tree.len()
    }

    pub fn is_top_cache_expired(&self, expire_len_secs: f64) -> bool {
        self.cache.top_cache_time.elapsed().as_secs_f64() > expire_len_secs
    }

    pub fn get_top(
        &mut self,
        count: usize,
        no_cache: bool,
        expire_len_secs: f64,
    ) -> Vec<(usize, Entry<K, V>)> {
        self.cache.top_requested = self.cache.top_requested || !no_cache;

        let cache_unusable = self.cache.top_cache.as_ref().is_none()
            || count <= self.cache.top_cache.as_ref().unwrap().len()
            || self.is_top_cache_expired(expire_len_secs);
        if no_cache || cache_unusable {
            let top = self.get_top_cacheless(count);
            if self.cache.top_requested && cache_unusable {
                self.cache.top_cache = Some(top.clone());
                self.cache.top_cache_time = Instant::now();
            }
            top
        } else {
            self.cache.top_cache.as_ref().unwrap()[0..count].to_vec()
        }
    }

    pub fn get_top_cacheless(&self, count: usize) -> Vec<(usize, Entry<K, V>)> {
        let mut ret = Vec::with_capacity(count);

        let mut cursor = self.tree.cursor();

        for _i in 0..count {
            cursor.move_prev();
            if let Some(v) = cursor.get_value() {
                let entry = v.clone();
                ret.push((cursor.get_index().unwrap() + 1, entry));
            } else {
                break;
            }
        }

        return ret;
    }

    pub fn is_bottom_cache_expired(&self, expire_len_secs: f64) -> bool {
        self.cache.bottom_cache_time.elapsed().as_secs_f64() > expire_len_secs
    }

    pub fn get_bottom(
        &mut self,
        count: usize,
        no_cache: bool,
        expire_len_secs: f64,
    ) -> Vec<(usize, Entry<K, V>)> {
        self.cache.bottom_requested = self.cache.bottom_requested || !no_cache;

        let cache_unusable = self.cache.bottom_cache.as_ref().is_none()
            || count <= self.cache.bottom_cache.as_ref().unwrap().len()
            || self.is_bottom_cache_expired(expire_len_secs);
        if no_cache || cache_unusable {
            let bottom = self.get_bottom_cacheless(count);
            if self.cache.bottom_requested && cache_unusable {
                self.cache.bottom_cache = Some(bottom.clone());
                self.cache.bottom_cache_time = Instant::now();
            }
            bottom
        } else {
            self.cache.bottom_cache.as_ref().unwrap()[0..count].to_vec()
        }
    }

    pub fn get_bottom_cacheless(&self, count: usize) -> Vec<(usize, Entry<K, V>)> {
        let mut ret = Vec::with_capacity(count);

        let mut cursor = self.tree.cursor();

        for _i in 0..count {
            cursor.move_next();
            if let Some(v) = cursor.get_value() {
                let entry = v.clone();
                ret.push((cursor.get_index().unwrap() + 1, entry));
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
            map: DiffMap::new(),
            size_cap: None,
            cache: Cache {
                top_cache: None,
                bottom_cache: None,
                top_cache_time: Instant::now(),
                bottom_cache_time: Instant::now(),
                top_requested: false,
                bottom_requested: false,
            },
        }
    }

    pub fn get_around(
        &self,
        id: &K,
        before: usize,
        after: usize,
    ) -> Option<Vec<(usize, Entry<K, V>)>> {
        let entry = self.map.get(id)?;
        let mut ret = Vec::with_capacity(before + after + 1);

        let mut cursor = self.tree.seek_val(&entry)?;
        let mut cursor2 = cursor.clone();

        for _i in 0..before {
            let val = cursor2.move_next();
            if let Some(v) = val {
                let entry = v.clone();
                ret.push((cursor2.get_index().unwrap() + 1, entry));
            } else {
                break;
            }
        }

        ret.reverse();

        ret.push((cursor.get_index()?, cursor.get_value().unwrap().clone()));

        for _i in 0..after {
            let val = cursor.move_prev();
            if let Some(v) = val {
                let entry = v.clone();
                ret.push((cursor.get_index().unwrap() + 1, entry));
            } else {
                break;
            }
        }

        ret.shrink_to_fit();

        return Some(ret);
    }

    pub fn get_after(&self, id: &K, count: usize) -> Option<Vec<(usize, Entry<K, V>)>> {
        let entry = self.map.get(id)?;
        let mut ret = Vec::with_capacity(count);

        let mut cursor = self.tree.seek_val(&entry).unwrap();
        for _i in 0..count {
            cursor.move_prev();
            if let Some(v) = cursor.get_value() {
                let entry = v.clone();
                ret.push((cursor.get_index().unwrap() + 1, entry));
            } else {
                break;
            }
        }

        ret.shrink_to_fit();

        Some(ret)
    }

    pub fn get_range(&self, start_rank: usize, end_rank: usize) -> Vec<(usize, Entry<K, V>)> {
        if end_rank < start_rank {
            return Vec::new();
        }
        let num = end_rank - start_rank + 1;
        let mut ret = Vec::with_capacity(num);

        let mut cursor = match self.tree.seek_index(start_rank - 1) {
            Some(v) => v,
            None => return Vec::new(),
        };
        for _i in 0..num {
            if let Some(v) = cursor.get_value() {
                let entry = v.clone();
                ret.push((cursor.get_index().unwrap() + 1, entry));
                cursor.move_prev();
            } else {
                break;
            }
        }

        ret.shrink_to_fit();

        return ret;
    }

    pub fn get_before(&self, id: &K, count: usize) -> Option<Vec<(usize, Entry<K, V>)>> {
        let entry = self.map.get(id)?;
        let mut ret = Vec::with_capacity(count);

        let mut cursor = self.tree.seek_val(&entry).unwrap();
        for _i in 0..count {
            cursor.move_next();
            if let Some(v) = cursor.get_value() {
                let entry = v.clone();
                ret.push((cursor.get_index().unwrap() + 1, entry));
            } else {
                break;
            }
        }

        ret.shrink_to_fit();

        Some(ret)
    }

    pub fn from_tree(tree: Tree<Entry<K, V>>) -> Self {
        let mut map = DiffMap::with_capacity(tree.len());

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
            size_cap: None,
            cache: Cache {
                top_cache: None,
                bottom_cache: None,
                top_cache_time: Instant::now(),
                bottom_cache_time: Instant::now(),
                top_requested: false,
                bottom_requested: false,
            },
        }
    }

    pub fn from_map(map: HashMap<K, Entry<K, V>>) -> Self {
        let mut tree = Tree::new();

        for (_, elem) in map.iter() {
            tree.insert(elem.clone());
        }

        Self {
            tree: tree,
            map: DiffMap::from_map(map),
            size_cap: None,
            cache: Cache {
                top_cache: None,
                bottom_cache: None,
                top_cache_time: Instant::now(),
                bottom_cache_time: Instant::now(),
                top_requested: false,
                bottom_requested: false,
            },
        }
    }

    pub fn from_map_prog(map: HashMap<K, Entry<K, V>>, prog: impl Fn(usize) -> ()) -> Self {
        let mut tree = Tree::new();

        for (_, elem) in map.iter() {
            tree.insert(elem.clone());
            prog(1);
        }

        Self {
            tree: tree,
            map: DiffMap::from_map(map),
            size_cap: None,
            cache: Cache {
                top_cache: None,
                bottom_cache: None,
                top_cache_time: Instant::now(),
                bottom_cache_time: Instant::now(),
                top_requested: false,
                bottom_requested: false,
            },
        }
    }

    pub fn get_min(&self) -> Option<V> {
        let mut c = self.tree.cursor();
        Some(c.move_next()?.points.clone())
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

impl<K, V, Context> Decode<Context> for Board<K, V>
where
    K: PartialOrd + Eq + Hash + Sized + Default + Clone + Decode<Context>,
    V: PartialOrd + Default + ?Sized + Clone + Decode<Context>,
{
    fn decode<D: bincode::de::Decoder>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError>
    where
        K: Decode<<D as Decoder>::Context>,
        V: Decode<<D as Decoder>::Context>,
    {
        let map = bincode::Decode::decode(decoder)?;

        Ok(Board::from_map(map))
    }
}

impl<K, V> Encode for Board<K, V>
where
    K: PartialOrd + Eq + Hash + Sized + Default + Clone + Encode,
    V: PartialOrd + Default + ?Sized + Clone + Encode,
{
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        bincode::Encode::encode(&self.map, encoder)?;

        Ok(())
    }
}
