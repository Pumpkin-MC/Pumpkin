use crate::serialization::{data_result::DataResult, dynamic_ops::DynamicOps};

/// A trait describing the way to encode something of a type `A` into something else  (`A -> ?`).
pub trait Encoder<A> {
    /// Encodes an input of this encoder's type (`A`) into an output of type `T`,
    /// along with a prefix (already encoded data).
    fn encode<T>(&self, input: A, ops: &impl DynamicOps<Value = T>, prefix: T) -> DataResult<T>;

    /// Encodes an input of this encoder's type (`A`) into an output of type `T`
    /// with no prefix (no already encoded data).
    fn encode_start<T>(&self, input: A, ops: &impl DynamicOps<Value = T>) -> DataResult<T> {
        return self.encode(input, ops, ops.empty())
    }
}