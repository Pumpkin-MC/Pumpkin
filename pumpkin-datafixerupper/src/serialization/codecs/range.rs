use crate::serialization::HasValue;
use crate::serialization::codec::Codec;
use crate::serialization::coders::{Decoder, Encoder};
use crate::serialization::data_result::DataResult;
use crate::serialization::dynamic_ops::DynamicOps;
use std::fmt::Display;

/// A codec for a specific number range.
/// - `C` is the type of codec used to serialize them (as if there was no range).
/// - `C::Value` (the codec type) is the type of number to restrict (by providing a range), while
pub struct RangeCodec<C: Codec + 'static>
where
    C::Value: PartialOrd + Display + Clone,
{
    codec: &'static C,
    min: C::Value,
    max: C::Value,
}

impl<C: Codec> HasValue for RangeCodec<C>
where
    <C as HasValue>::Value: PartialOrd + Display + Clone,
{
    type Value = C::Value;
}

impl<C: Codec> Encoder for RangeCodec<C>
where
    <C as HasValue>::Value: PartialOrd + Display + Clone,
{
    fn encode<T: Display + PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T> {
        check_range(input, &self.min, &self.max).flat_map(|t| self.codec.encode(&t, ops, prefix))
    }
}

impl<C: Codec> Decoder for RangeCodec<C>
where
    <C as HasValue>::Value: PartialOrd + Display + Clone,
{
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(Self::Value, T)> {
        self.codec
            .decode(input, ops)
            .flat_map(|(i, t)| check_range(&i, &self.min, &self.max).map(|n| (n, t)))
    }
}

/// A helper function to check whether a number is between the range `[min, max]` (both inclusive).
fn check_range<T: PartialOrd + Display + Clone>(input: &T, min: &T, max: &T) -> DataResult<T> {
    if input >= min && input <= max {
        DataResult::success(input.clone())
    } else {
        DataResult::error(format!("Value {input} is outside range [{min}, {max}]"))
    }
}

pub(crate) const fn new_range_codec<A: Display + PartialOrd + Clone, C: Codec<Value = A>>(
    codec: &'static C,
    min: A,
    max: A,
) -> RangeCodec<C> {
    RangeCodec { codec, min, max }
}
