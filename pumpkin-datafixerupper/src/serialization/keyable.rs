use crate::serialization::dynamic_ops::DynamicOps;

/// A trait that specifies that an object can be represented with keys, like maps or `struct` types.
pub trait Keyable {
    /// Returns an iterator over the keys of this `Keyable`.
    fn iter_keys<T>(
        &self,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> impl Iterator<Item = T> + '_;
}
