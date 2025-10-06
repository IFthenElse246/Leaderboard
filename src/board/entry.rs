use std::cmp;

use bincode::{Decode, Encode, de::Decoder};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Serialize, Deserialize)]
pub struct Entry<K, V>
where
    K: PartialOrd + Default,
    V: PartialOrd + Sized + Default,
{
    pub timestamp: f64,
    pub points: V,
    pub key: K,
}

impl<K: PartialOrd + Default, V: PartialOrd + Sized + Default> PartialOrd for Entry<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.points.partial_cmp(&other.points) {
            Some(cmp::Ordering::Equal) => match other.timestamp.partial_cmp(&self.timestamp) {
                Some(cmp::Ordering::Equal) => other.key.partial_cmp(&self.key),
                v => v,
            },
            v => v,
        }
    }
}

impl<K: PartialOrd + Default, V: PartialOrd + Sized + Default> Ord for Entry<K, V> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.points.partial_cmp(&other.points) {
            Some(cmp::Ordering::Equal) | None => match other.timestamp.partial_cmp(&self.timestamp) {
                None | Some(cmp::Ordering::Equal) => match other.key.partial_cmp(&self.key) {
                    None => cmp::Ordering::Equal,
                    Some(v) => v,
                },
                Some(v) => v,
            },
            Some(v) => v,
        }
    }
}

impl<K: PartialOrd + Default, V: PartialOrd + Sized + Default> Default for Entry<K, V> {
    fn default() -> Self {
        Self {
            timestamp: 0 as f64,
            points: V::default(),
            key: K::default(),
        }
    }
}

impl<K: PartialOrd + Default, V: PartialOrd + Sized + Default> Eq for Entry<K, V> {}
impl<K: PartialOrd + Default + Clone, V: PartialOrd + Sized + Default + Clone> Clone
    for Entry<K, V>
{
    fn clone(&self) -> Self {
        Self {
            timestamp: self.timestamp.clone(),
            points: self.points.clone(),
            key: self.key.clone(),
        }
    }
}

impl<
    K: PartialOrd + Default + Decode<Context>,
    V: PartialOrd + Sized + Default + Decode<Context>,
    Context,
> Decode<Context> for Entry<K, V>
{
    fn decode<D: bincode::de::Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError>
    where
        K: Decode<<D as Decoder>::Context>,
        V: Decode<<D as Decoder>::Context>,
    {
        Ok(Self {
            key: bincode::Decode::decode(decoder)?,
            timestamp: bincode::Decode::decode(decoder)?,
            points: bincode::Decode::decode(decoder)?,
        })
    }
}

impl<K: PartialOrd + Default + Encode, V: PartialOrd + Sized + Default + Encode> Encode
    for Entry<K, V>
{
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        bincode::Encode::encode(&self.key, encoder)?;
        bincode::Encode::encode(&self.timestamp, encoder)?;
        bincode::Encode::encode(&self.points, encoder)?;
        Ok(())
    }
}
