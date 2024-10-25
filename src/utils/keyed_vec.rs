use std::iter::Zip;

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
    pub fn add(&mut self, key: K, value: V) {
        let idx = self.keys.iter().position(|k| *k == key);
        match idx {
            Some(_) => panic!("Key already exists!"),
            None => {}
        };
        self.keys.push(key);
        self.values.push(value);
    }

    pub(crate) fn get(&self, key: &K) -> &V {
        let idx = self
            .keys
            .iter()
            .position(|k| *k == *key)
            .expect("Key not found!");
        &self.values[idx]
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
