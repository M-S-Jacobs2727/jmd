use std::iter::Zip;

use crate::Error;

#[derive(Debug)]
pub struct KeyedVec<K, V>
where
    K: PartialEq,
{
    keys: Vec<K>,
    values: Vec<V>,
}
impl<K, V> KeyedVec<K, V>
where
    K: PartialEq,
{
    pub fn new() -> Self {
        Self {
            keys: Vec::new(),
            values: Vec::new(),
        }
    }
    pub fn add(&mut self, key: K, value: V) -> Result<(), Error> {
        let idx = self.keys.iter().position(|k| *k == key);
        if let Some(_) = idx {
            return Err(Error::OtherError);
        }
        self.keys.push(key);
        self.values.push(value);
        Ok(())
    }

    pub(crate) fn get(&self, key: &K) -> Result<&V, Error> {
        let idx = self.keys.iter().position(|k| *k == *key);
        match idx {
            Some(i) => Ok(&self.values[i]),
            None => Err(Error::OtherError),
        }
    }
    pub(crate) fn index(&self, idx: usize) -> &V {
        &self.values[idx]
    }
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub(crate) fn contains(&self, key: &K) -> bool {
        self.keys.contains(key)
    }
}

impl<K, V> IntoIterator for KeyedVec<K, V>
where
    K: PartialEq,
{
    type IntoIter = Zip<std::vec::IntoIter<K>, std::vec::IntoIter<V>>;
    type Item = (K, V);
    fn into_iter(self) -> Self::IntoIter {
        self.keys.into_iter().zip(self.values.into_iter())
    }
}
