use std::cmp;

#[derive(PartialEq)]
pub struct Entry<V>
where
    V: PartialOrd + ?Sized,
{
    pub timestamp: u128,
    pub points: V,
}

impl<V: PartialOrd + ?Sized> PartialOrd for Entry<V> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        if self.points != other.points {
            return self.points.partial_cmp(&other.points);
        }
        return other.timestamp.partial_cmp(&self.timestamp);
    }
}

impl<V: PartialOrd + ?Sized > Eq for Entry<V> {}
impl<V: PartialOrd + ?Sized + Clone> Clone for Entry<V> {
    fn clone(&self) -> Self {
        Self {
            timestamp: self.timestamp.clone(),
            points: self.points.clone()
        }
    }
}