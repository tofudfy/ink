// Copyright 2019-2020 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A storage hash map that allows to associate keys with values.

mod impls;
mod iter;
mod storage;

#[cfg(test)]
mod tests;

#[cfg(all(test, feature = "ink-fuzz-tests"))]
mod fuzz_tests;

pub use self::iter::{
    Iter,
    IterMut,
    Keys,
    Values,
    ValuesMut,
};
use crate::{
    hash::hasher::{
        Blake2x256Hasher,
        Hasher,
    },
    storage2::{
        collections::Stash,
        lazy::LazyHashMap,
        traits::PackedLayout,
    },
};
use core::{
    borrow::Borrow,
    cmp::Eq,
};
use ink_prelude::borrow::ToOwned;
use ink_primitives::Key;

/// The index type within a hashmap.
///
/// # Note
///
/// Used for key indices internal to the hashmap.
type KeyIndex = u32;

/// A hash map operating on the contract storage.
///
/// Stores a mapping between keys and values.
///
/// # Note
///
/// Unlike Rust's standard `HashMap` that uses the [`core::hash::Hash`] trait
/// in order to hash its keys the storage hash map uses the [`scale::Encode`]
/// encoding in order to hash its keys using a built-in cryptographic
/// hash function provided by the chain runtime.
///
/// The main difference between the lower-level `LazyHashMap` and the
/// `storage::HashMap` is that the latter is aware of its associated keys and
/// values and operates on those instances directly as opposed to `Option`
/// instances of them. Also it provides a more high-level and user focused
/// API.
///
/// Users should generally prefer using this storage hash map over the low-level
/// `LazyHashMap` for direct usage in their smart contracts.
#[derive(Debug)]
pub struct HashMap<K, V, H = Blake2x256Hasher>
where
    K: Ord + Clone + PackedLayout,
    V: PackedLayout,
    H: Hasher,
    Key: From<<H as Hasher>::Output>,
{
    /// The keys of the storage hash map.
    keys: Stash<K>,
    /// The values of the storage hash map.
    values: LazyHashMap<K, ValueEntry<V>, H>,
}

/// An entry within the storage hash map.
///
/// Stores the value as well as the index to its associated key.
#[derive(Debug, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
struct ValueEntry<V> {
    /// The value stored in this entry.
    value: V,
    /// The index of the key associated with this value.
    key_index: KeyIndex,
}

/// A vacant entry with previous and next vacant indices.
pub struct OccupiedEntry<'a, K, V, H = Blake2x256Hasher>
where
    K: Ord + Clone + PackedLayout,
    V: PackedLayout,
    H: Hasher,
    Key: From<<H as Hasher>::Output>,
{
    /// A reference to the used `HashMap` instance.
    base: &'a mut HashMap<K, V, H>,
    /// The index of the key associated with this value.
    key_index: KeyIndex,
    /// The key stored in this entry.
    key: K,
}

/// A vacant entry with previous and next vacant indices.
pub struct VacantEntry<'a, K, V, H = Blake2x256Hasher>
where
    K: Ord + Clone + PackedLayout,
    V: PackedLayout,
    H: Hasher,
    Key: From<<H as Hasher>::Output>,
{
    /// A reference to the used `HashMap` instance.
    base: &'a mut HashMap<K, V, H>,
    /// The key stored in this entry.
    key: K,
}

/// An entry within the stash.
///
/// The vacant entries within a storage stash form a doubly linked list of
/// vacant entries that is used to quickly re-use their vacant storage.
pub enum Entry<'a, K: 'a, V: 'a, H = Blake2x256Hasher>
where
    K: Ord + Clone + PackedLayout,
    V: PackedLayout,
    H: Hasher,
    Key: From<<H as Hasher>::Output>,
{
    /// A vacant entry that holds the index to the next and previous vacant entry.
    Vacant(VacantEntry<'a, K, V, H>),
    /// An occupied entry that holds the value.
    Occupied(OccupiedEntry<'a, K, V, H>),
}

impl<K, V, H> HashMap<K, V, H>
where
    K: Ord + Clone + PackedLayout,
    V: PackedLayout,
    H: Hasher,
    Key: From<<H as Hasher>::Output>,
{
    /// Creates a new empty storage hash map.
    pub fn new() -> Self {
        Self {
            keys: Stash::new(),
            values: LazyHashMap::new(),
        }
    }

    /// Returns the number of key- value pairs stored in the hash map.
    pub fn len(&self) -> u32 {
        self.keys.len()
    }

    /// Returns `true` if the hash map is empty.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Returns an iterator yielding shared references to all key/value pairs
    /// of the hash map.
    ///
    /// # Note
    ///
    /// - Avoid unbounded iteration over big storage hash maps.
    /// - Prefer using methods like `Iterator::take` in order to limit the number
    ///   of yielded elements.
    pub fn iter(&self) -> Iter<K, V, H> {
        Iter::new(self)
    }

    /// Returns an iterator yielding exclusive references to all key/value pairs
    /// of the hash map.
    ///
    /// # Note
    ///
    /// - Avoid unbounded iteration over big storage hash maps.
    /// - Prefer using methods like `Iterator::take` in order to limit the number
    ///   of yielded elements.
    pub fn iter_mut(&mut self) -> IterMut<K, V, H> {
        IterMut::new(self)
    }

    /// Returns an iterator yielding shared references to all values of the hash map.
    ///
    /// # Note
    ///
    /// - Avoid unbounded iteration over big storage hash maps.
    /// - Prefer using methods like `Iterator::take` in order to limit the number
    ///   of yielded elements.
    pub fn values(&self) -> Values<K, V, H> {
        Values::new(self)
    }

    /// Returns an iterator yielding shared references to all values of the hash map.
    ///
    /// # Note
    ///
    /// - Avoid unbounded iteration over big storage hash maps.
    /// - Prefer using methods like `Iterator::take` in order to limit the number
    ///   of yielded elements.
    pub fn values_mut(&mut self) -> ValuesMut<K, V, H> {
        ValuesMut::new(self)
    }

    /// Returns an iterator yielding shared references to all keys of the hash map.
    ///
    /// # Note
    ///
    /// - Avoid unbounded iteration over big storage hash maps.
    /// - Prefer using methods like `Iterator::take` in order to limit the number
    ///   of yielded elements.
    pub fn keys(&self) -> Keys<K> {
        Keys::new(self)
    }
}

impl<K, V, H> HashMap<K, V, H>
where
    K: Ord + Clone + PackedLayout,
    V: PackedLayout,
    H: Hasher,
    Key: From<<H as Hasher>::Output>,
{
    fn clear_cells(&self) {
        if self.values.key().is_none() {
            // We won't clear any storage if we are in lazy state since there
            // probably has not been any state written to storage, yet.
            return
        }
        for key in self.keys() {
            // It might seem wasteful to clear all entries instead of just
            // the occupied ones. However this spares us from having one extra
            // read for every element in the storage stash to filter out vacant
            // entries. So this is actually a trade-off and at the time of this
            // implementation it is unclear which path is more efficient.
            //
            // The bet is that clearing a storage cell is cheaper than reading one.
            self.values.clear_packed_at(key);
        }
    }
}

impl<K, V, H> HashMap<K, V, H>
where
    K: Ord + Eq + Clone + PackedLayout,
    V: PackedLayout,
    H: Hasher,
    Key: From<H::Output>,
{
    /// Inserts a key-value pair into the map.
    ///
    /// Returns the previous value associated with the same key if any.
    /// If the map did not have this key present, `None` is returned.
    ///
    /// # Note
    ///
    /// - If the map did have this key present, the value is updated,
    ///   and the old value is returned. The key is not updated, though;
    ///   this matters for types that can be `==` without being identical.
    pub fn insert(&mut self, key: K, new_value: V) -> Option<V> {
        if let Some(occupied) = self.values.get_mut(&key) {
            // Update value, don't update key.
            let old_value = core::mem::replace(&mut occupied.value, new_value);
            return Some(old_value)
        }
        // At this point we know that `key` does not yet exist in the map.
        let key_index = self.keys.put(key.to_owned());
        self.values.put(
            key,
            Some(ValueEntry {
                value: new_value,
                key_index,
            }),
        );
        None
    }

    /// Removes the key/value pair from the map associated with the given key.
    ///
    /// - Returns the removed value if any.
    ///
    /// # Note
    ///
    /// The key may be any borrowed form of the map's key type,
    /// but `Hash` and `Eq` on the borrowed form must match those for the key type.
    pub fn take<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + scale::Encode + ToOwned<Owned = K>,
    {
        let entry = self.values.put_get(key, None)?;
        self.keys
            .take(entry.key_index)
            .expect("`key_index` must point to a valid key entry");
        Some(entry.value)
    }

    /// Returns a shared reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type,
    /// but `Hash` and `Eq` on the borrowed form must match those for the key type.
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + scale::Encode + ToOwned<Owned = K>,
    {
        self.values.get(key).map(|entry| &entry.value)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type,
    /// but `Hash` and `Eq` on the borrowed form must match those for the key type.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + scale::Encode + ToOwned<Owned = K>,
    {
        self.values.get_mut(key).map(|entry| &mut entry.value)
    }

    /// Returns `true` if there is an entry corresponding to the key in the map.
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + PartialEq<K> + Eq + scale::Encode + ToOwned<Owned = K>,
    {
        // We do not check if the given key is equal to the queried key which is
        // what normally a hash map implementation does because we do not resolve
        // or prevent collisions in this hash map implementation at any level.
        // Having a collision is virtually impossible since we
        // are using a keyspace of 2^256 bit.
        self.values.get(key).is_some()
    }

    /// Defragments storage used by the storage hash map.
    ///
    /// Returns the number of storage cells freed this way.
    ///
    /// A `max_iterations` parameter of `None` means that there is no limit
    /// to the number of iterations performed. This is generally not advised.
    ///
    /// # Note
    ///
    /// This frees storage that is held but not necessary for the hash map to hold.
    /// This operation might be expensive, especially for big `max_iteration`
    /// parameters. The `max_iterations` parameter can be used to limit the
    /// expensiveness for this operation and instead free up storage incrementally.
    pub fn defrag(&mut self, max_iterations: Option<u32>) -> u32 {
        // This method just defrags the underlying `storage::Stash` used to
        // store the keys as it can sometimes take a lot of unused storage
        // if many keys have been removed at some point. Some hash map
        // implementations might even prefer to perform this operation with a
        // limit set to 1 after every successful removal.
        if let Some(0) = max_iterations {
            // Bail out early if the iteration limit is set to 0 anyways to
            // completely avoid doing work in this case.y
            return 0
        }
        let len_vacant = self.keys.capacity() - self.keys.len();
        let max_iterations = max_iterations.unwrap_or(len_vacant);
        let values = &mut self.values;
        let callback = |old_index, new_index, key: &K| {
            let value_entry = values.get_mut(key).expect("key must be valid");
            debug_assert_eq!(value_entry.key_index, old_index);
            value_entry.key_index = new_index;
        };
        self.keys.defrag(Some(max_iterations), callback)
    }

    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    pub fn entry(&mut self, key: K) -> Entry<K, V, H> {
        let v = self.values.get(&key);
        match v {
            Some(entry) => {
                Entry::Occupied(OccupiedEntry {
                    key,
                    key_index: entry.key_index,
                    base: self,
                })
            }
            None => Entry::Vacant(VacantEntry { key, base: self }),
        }
    }
}

impl<'a, K, V, H> Entry<'a, K, V, H>
where
    K: Ord + Clone + PackedLayout,
    V: PackedLayout + core::fmt::Debug + core::cmp::Eq + Default,
    H: Hasher,
    Key: From<<H as Hasher>::Output>,
{
    /// Returns a reference to this entry's key.
    pub fn key(&self) -> &K {
        match self {
            Entry::Occupied(entry) => &entry.key,
            Entry::Vacant(entry) => &entry.key,
        }
    }

    /// Ensures a value is in the entry by inserting the default value if empty, and returns
    /// a reference to the value in the entry.
    pub fn or_default(self) -> &'a V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(V::default()),
        }
    }

    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry.
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default function if empty,
    /// and returns mutable references to the key and value in the entry.
    pub fn or_insert_with<F>(self, default: F) -> &'a mut V
    where
        F: FnOnce() -> V,
    {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => Entry::insert(default(), entry),
        }
    }

    /// Ensures a value is in the entry by inserting, if empty, the result of the default
    /// function, which takes the key as its argument, and returns a mutable reference to
    /// the value in the entry.
    pub fn or_insert_with_key<F>(self, default: F) -> &'a mut V
    where
        F: FnOnce(&K) -> V,
    {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => Entry::insert(default(&entry.key), entry),
        }
    }

    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the map.
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        match self {
            Entry::Occupied(mut entry) => {
                {
                    let v = entry.get_mut();
                    f(v);
                }
                Entry::Occupied(entry)
            }
            Entry::Vacant(entry) => Entry::Vacant(entry),
        }
    }

    /// Inserts `value` into `entry`.
    fn insert(value: V, entry: VacantEntry<'a, K, V, H>) -> &'a mut V {
        let old_value = entry.base.insert(entry.key.clone(), value);
        debug_assert!(old_value.is_none());
        entry
            .base
            .get_mut(&entry.key)
            .expect("encountered invalid vacant entry")
    }
}

impl<'a, K, V, H> VacantEntry<'a, K, V, H>
where
    K: Ord + Clone + PackedLayout,
    V: PackedLayout,
    H: Hasher,
    Key: From<<H as Hasher>::Output>,
{
    /// Gets a reference to the key that would be used when inserting a value through the VacantEntry.
    pub fn key(&self) -> &K {
        &self.key
    }

    /// Take ownership of the key.
    pub fn into_key(self) -> K {
        self.key
    }

    /// Sets the value of the entry with the VacantEntry's key, and returns a mutable reference to it.
    pub fn insert(self, value: V) -> &'a mut V {
        // At this point we know that `key` does not yet exist in the map.
        let key_index = self.base.keys.put(self.key.clone());
        self.base
            .values
            .put(self.key.clone(), Some(ValueEntry { value, key_index }));
        self.base
            .get_mut(&self.key)
            .expect("put was just executed; qed")
    }
}

impl<'a, K, V, H> OccupiedEntry<'a, K, V, H>
where
    K: Ord + Clone + PackedLayout,
    V: PackedLayout,
    H: Hasher,
    Key: From<<H as Hasher>::Output>,
{
    /// Gets a reference to the key in the entry.
    pub fn key(&self) -> &K {
        &self.key
    }

    /// Take the ownership of the key and value from the map.
    pub fn remove_entry(self) -> (K, V) {
        let entry = self
            .base
            .values
            .put_get(&self.key, None)
            .expect("`key` must exist");
        self.base
            .keys
            .take(self.key_index)
            .expect("`key_index` must point to a valid key entry");
        (self.key, entry.value)
    }

    /// Gets a reference to the value in the entry.
    pub fn get(&self) -> &V {
        &self
            .base
            .get(&self.key)
            .expect("entry behind `OccupiedEntry` must always exist")
    }

    /// Gets a mutable reference to the value in the entry.
    ///
    /// If you need a reference to the `OccupiedEntry` which may outlive the destruction of the
    /// `Entry` value, see `into_mut`.
    pub fn get_mut(&mut self) -> &mut V {
        self.base
            .get_mut(&self.key)
            .expect("entry behind `OccupiedEntry` must always exist")
    }

    /// Sets the value of the entry, and returns the entry's old value.
    pub fn insert(&mut self, new_value: V) -> V {
        let occupied = self
            .base
            .values
            .get_mut(&self.key)
            .expect("entry behind `OccupiedEntry` must always exist");
        core::mem::replace(&mut occupied.value, new_value)
    }

    /// Takes the value out of the entry, and returns it.
    pub fn remove(self) -> V {
        self.remove_entry().1
    }

    /// Converts the OccupiedEntry into a mutable reference to the value in the entry
    /// with a lifetime bound to the map itself.
    pub fn into_mut(self) -> &'a mut V {
        self.base
            .get_mut(&self.key)
            .expect("entry behind `OccupiedEntry` must always exist")
    }
}