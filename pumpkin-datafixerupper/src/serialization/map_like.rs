use std::fmt::Display;

use crate::serialization::dynamic_ops::DynamicOps;

/// A trait that can be used to treat something as a map view of a dynamic type.
/// The [`Value`] of this trait is the *dynamic type* of this map-like.
pub trait MapLike: Display {
    type Value;

    /// Gets the value of this map view with a key of the *dynamic type* of this map-like.
    fn get(&self, key: &Self::Value) -> Option<&Self::Value>;

    /// Gets the value of this map view with a `&str` key of the *dynamic type* of this map-like with the provided [`DynamicOps`] of this map-like's *dynamic type*.
    fn get_str<O: DynamicOps<Value = Self::Value>>(
        &self,
        key: &str,
        ops: &O,
    ) -> Option<&Self::Value>;

    /// Returns an `Iterator` to each key-value pair in this map-like, both of its *dynamic type*.
    fn entries(&self) -> impl Iterator<Item = (&Self::Value, &Self::Value)>;
}
