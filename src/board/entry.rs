use std::cmp;

#[derive(PartialEq)]
pub struct Entry<K, V>
where
    K: PartialOrd + Default,
    V: PartialOrd + Sized + Default,
{
    pub timestamp: u128,
    pub points: V,
    pub key: K
}

impl<K: PartialOrd + Default, V: PartialOrd + Sized + Default> PartialOrd for Entry<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.points.partial_cmp(&other.points) {
            Some(cmp::Ordering::Equal) => match other.timestamp.partial_cmp(&self.timestamp) {
                Some(cmp::Ordering::Equal) => other.key.partial_cmp(&self.key),
                v => v
            },
            v => v
        }
    }
}

impl<K: PartialOrd + Default, V: PartialOrd + Sized + Default> Ord for Entry<K, V> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.points.partial_cmp(&other.points) {
            Some(cmp::Ordering::Equal) | None => match other.timestamp.cmp(&self.timestamp) {
                cmp::Ordering::Equal => match other.key.partial_cmp(&self.key) {
                    None => cmp::Ordering::Equal,
                    Some(v) => v
                },
                v => v
            },
            Some(v) => v
        }
    }
}

impl<K: PartialOrd + Default, V: PartialOrd + Sized + Default> Default for Entry<K, V> {
    fn default() -> Self {
        Self {
            timestamp: 0,
            points: V::default(),
            key: K::default()
        }
    }
}

impl<K: PartialOrd + Default, V: PartialOrd + Sized + Default> Eq for Entry<K, V> {}
impl<K: PartialOrd + Default + Clone, V: PartialOrd + Sized + Default + Clone> Clone for Entry<K, V> {
    fn clone(&self) -> Self {
        Self {
            timestamp: self.timestamp.clone(),
            points: self.points.clone(),
            key: self.key.clone()
        }
    }
}