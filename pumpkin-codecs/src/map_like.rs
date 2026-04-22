use std::iter;
use std::marker::PhantomData;

/// A trait that can be used to treat something as a map view of a dynamic type.
/// The [`Value`] of this trait is the *dynamic type* of this map-like.
pub trait MapLike {
    type Value;

    /// Gets the value of this map view with a key of the *dynamic type* of this map-like.
    fn get(&self, key: &Self::Value) -> Option<&Self::Value>;

    /// Gets the value of this map view with a `&str` key of the *dynamic type* of this map-like with the provided [`DynamicOps`] of this map-like's *dynamic type*.
    fn get_str(&self, key: &str) -> Option<&Self::Value>;

    /// Returns an `Iterator` to each key-value pair in this map-like, both of its *dynamic type*.
    fn iter(&self) -> impl Iterator<Item = (Self::Value, &Self::Value)> + '_;
}

/// A [`MapLike`] implementation that is empty.
pub struct EmptyMapLike<T> {
    phantom: PhantomData<T>,
}

impl<T> MapLike for EmptyMapLike<T> {
    type Value = T;

    fn get(&self, _key: &Self::Value) -> Option<&Self::Value> {
        None
    }

    fn get_str(&self, _key: &str) -> Option<&Self::Value> {
        None
    }

    fn iter(&self) -> impl Iterator<Item = (Self::Value, &Self::Value)> + '_ {
        iter::empty()
    }
}

impl<T> EmptyMapLike<T> {
    /// Constructs a new empty `MapLike`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<T> Default for EmptyMapLike<T> {
    fn default() -> Self {
        Self::new()
    }
}
