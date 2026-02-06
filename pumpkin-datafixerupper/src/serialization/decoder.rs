use crate::serialization::{data_result::DataResult, dynamic_ops::DynamicOps};

/// A trait describing the way to decode something of type into something of type `A` (`? -> A`).
pub trait Decoder<A> {
    /// Decodes an input of this decoder's type (`A`) into an output of type `T`,
    /// keeping the remaining undecoded data as another element of the tuple.
    fn decode<T>(&self, input: A, ops: &impl DynamicOps<Value = T>) -> DataResult<(A, T)>;

    /// Decodes an input of this decoder's type (`A`) into an output of type `T`,
    /// ignoring any remaining undecoded data (of type `A`).
    fn read<T>(&self, input: A, ops: &impl DynamicOps<Value = T>) -> DataResult<A> {
        return self.decode(input, ops).map(|r| r.0)
    }
}