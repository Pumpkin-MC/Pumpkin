/// A trait that specifies that an object can be represented with keys, like maps or `struct` types.
pub trait Keyable {
    /// Returns a new copy of a [`Vec`] of the keys of this `Keyable`.
    #[must_use]
    fn iter_keys(&self) -> Vec<String>;
}
