use std::collections::HashMap;

use crate::serialization::dynamic_ops::DynamicOps;

/// A trait that can be used to treat something as a map view of a dynamic type.
/// The [`Value`] of this trait is the *dynamic type* of this map-like.
pub trait MapLike<'a> {
    type Value;

    /// Gets the value of this map view with a key of the *dynamic type* of this map-like.
    fn get(&self, key: Self::Value) -> Option<&Self::Value>;

    /// Gets the value of this map view with a `&str` key of the *dynamic type* of this map-like with the provided [`DynamicOps`] of this map-like's *dynamic type*.
    fn get_str<O: DynamicOps<Value = Self::Value>>(
        &self,
        key: &str,
        ops: &O,
    ) -> Option<&Self::Value>;

    /// Returns an `Iterator` to each key-value pair in this map-like, both of its *dynamic type*.
    fn entries(&self) -> Box<dyn Iterator<Item = (&'a Self::Value, &'a Self::Value)> + 'a>;

    /// Returns a map-like view of a [`HashMap`] in Rust.
    #[must_use]
    fn for_map<V, T: DynamicOps<Value = Self::Value>>(
        map: &'a HashMap<V, V>,
    ) -> impl MapLike<'a> + 'a
    where
        V: Eq + std::hash::Hash,
    {
        MapLikeFromMap { map }
    }
}

/// An implementation of [`MapLike`] for a [`HashMap`] in Rust.
pub struct MapLikeFromMap<'a, V> {
    map: &'a HashMap<V, V>,
}

impl<'a, V> MapLike<'a> for MapLikeFromMap<'a, V>
where
    V: Eq + std::hash::Hash,
{
    type Value = V;

    fn get(&self, key: V) -> Option<&V> {
        self.map.get(&key)
    }

    fn get_str<O: DynamicOps<Value = V>>(&self, key: &str, ops: &O) -> Option<&V> {
        self.map.get(&ops.create_string(key.to_owned()))
    }

    fn entries(&self) -> Box<dyn Iterator<Item = (&'a V, &'a V)> + 'a> {
        Box::new(self.map.iter())
    }
}
