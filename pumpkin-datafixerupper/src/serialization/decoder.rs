use crate::serialization::{data_result::DataResult, dynamic_ops::DynamicOps};

/// A trait describing the way to decode something of type into something of type `A` (`? -> A`).
pub trait Decoder<A>: 'static {
    /// Decodes an input of this decoder's type (`A`) into an output of type `T`,
    /// keeping the remaining undecoded data as another element of the tuple.
    fn decode<T: PartialEq>(&self, input: T, ops: &impl DynamicOps<Value = T>) -> DataResult<(A, T)>;

    /// Decodes an input of this decoder's type (`A`) into an output of type `T`,
    /// ignoring any remaining undecoded data (of type `A`).
    fn parse<T: PartialEq>(&self, input: T, ops: &impl DynamicOps<Value = T>) -> DataResult<A> {
        self.decode(input, ops).map(|r| r.0)
    }
}
