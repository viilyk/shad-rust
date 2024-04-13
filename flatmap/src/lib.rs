#![forbid(unsafe_code)]

use std::mem::replace;
use std::{borrow::Borrow, iter::FromIterator, ops::Index};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug, PartialEq, Eq)]
pub struct FlatMap<K, V>(Vec<(K, V)>);

impl<K: Ord, V> FlatMap<K, V> {
    pub fn new() -> Self {
        FlatMap(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    pub fn as_slice(&self) -> &[(K, V)] {
        self.0.as_slice()
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // Returns None if key was not present, or Some(prev_value) if it was.
        let ind = self.0.binary_search_by(|pair| pair.0.cmp(&key));
        match ind {
            Ok(x) => Some(replace(&mut self.0[x].1, value)),
            Err(x) => {
                self.0.insert(x, (key, value));
                None
            }
        }
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let ind = self.0.binary_search_by(|pair| pair.0.borrow().cmp(key));
        match ind {
            Ok(x) => Some(&self.0[x].1),
            Err(_) => None,
        }
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let ind = self.0.binary_search_by(|pair| pair.0.borrow().cmp(key));
        match ind {
            Ok(x) => Some(self.0.remove(x).1),
            Err(_) => None,
        }
    }

    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let ind = self.0.binary_search_by(|pair| pair.0.borrow().cmp(key));
        match ind {
            Ok(x) => Some(self.0.remove(x)),
            Err(_) => None,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<Q: ?Sized + Ord, K: Ord + Borrow<Q>, V> Index<&Q> for FlatMap<K, V> {
    type Output = V;

    fn index(&self, index: &Q) -> &Self::Output {
        let ind = self.0.binary_search_by(|pair| pair.0.borrow().cmp(index));
        match ind {
            Ok(x) => &self.0[x].1,
            Err(_) => panic!("aaaa"),
        }
    }
}

impl<K: Ord, V> Extend<(K, V)> for FlatMap<K, V> {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (K, V)>,
    {
        for (key, value) in iter {
            self.insert(key, value);
        }
    }
}

impl<K: Ord, V> From<Vec<(K, V)>> for FlatMap<K, V> {
    fn from(mut value: Vec<(K, V)>) -> Self {
        value.sort_by(|a, b| a.0.cmp(&b.0));
        value.reverse();
        value.dedup_by(|a, b| a.0.eq(&b.0));
        value.reverse();
        FlatMap(value)
    }
}

impl<K: Ord, V> From<FlatMap<K, V>> for Vec<(K, V)> {
    fn from(value: FlatMap<K, V>) -> Self {
        value.0
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for FlatMap<K, V> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (K, V)>,
    {
        let mut c = FlatMap::new();
        for (key, value) in iter {
            c.insert(key, value);
        }
        c
    }
}

impl<K: Ord, V> IntoIterator for FlatMap<K, V> {
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
