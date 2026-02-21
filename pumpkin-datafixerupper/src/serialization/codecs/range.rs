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

#[cfg(test)]
mod test {
    use crate::serialization::codec::*;
    use crate::serialization::coders::*;
    use crate::serialization::json_ops;
    use crate::{assert_decode, assert_success};
    use serde_json::json;

    #[test]
    fn encoding() {
        {
            // A codec that does not allow negative numbers.
            pub static NON_NEGATIVE_INT_CODEC: IntRangeCodec = int_range(0, i32::MAX);

            assert_success!(
                NON_NEGATIVE_INT_CODEC.encode_start(&3, &json_ops::INSTANCE),
                json!(3)
            );
            assert_success!(
                NON_NEGATIVE_INT_CODEC.encode_start(&6745, &json_ops::INSTANCE),
                json!(6745)
            );
            assert_success!(
                NON_NEGATIVE_INT_CODEC.encode_start(&0, &json_ops::INSTANCE),
                json!(0)
            );
            assert!(
                NON_NEGATIVE_INT_CODEC
                    .encode_start(&-93, &json_ops::INSTANCE)
                    .is_error()
            );
        };

        {
            // A codec accepting a double value from 0 to 100.
            pub static PERCENTAGE_CODEC: DoubleRangeCodec = double_range(0.0, 100.0);

            assert!(
                PERCENTAGE_CODEC
                    .encode_start(&16.0, &json_ops::INSTANCE)
                    .is_success()
            );
            assert!(
                PERCENTAGE_CODEC
                    .encode_start(&45.5, &json_ops::INSTANCE)
                    .is_success()
            );
            assert!(
                PERCENTAGE_CODEC
                    .encode_start(&99.999, &json_ops::INSTANCE)
                    .is_success()
            );
            assert!(
                PERCENTAGE_CODEC
                    .encode_start(&134.4, &json_ops::INSTANCE)
                    .is_error()
            );
        };
    }

    #[test]
    fn decoding() {
        assert_decode!(int_range(1, 5), json!(3), &json_ops::INSTANCE, is_success);
        assert_decode!(int_range(-5, 5), json!(6), &json_ops::INSTANCE, is_error);

        assert_decode!(
            double_range(-100.0, 100.0),
            json!(45.5),
            &json_ops::INSTANCE,
            is_success
        );
        assert_decode!(
            double_range(-100.0, 100.0),
            json!(-100),
            &json_ops::INSTANCE,
            is_success
        );
        assert_decode!(
            double_range(1.0, f64::MAX),
            json!(88.44),
            &json_ops::INSTANCE,
            is_success
        );
        assert_decode!(
            double_range(1.0, f64::MAX),
            json!(0.999),
            &json_ops::INSTANCE,
            is_error
        );

        assert_decode!(
            float_range(0.04, 0.08),
            json!(0.05),
            &json_ops::INSTANCE,
            is_success
        );
        assert_decode!(
            float_range(0.006, 0.012),
            json!(0.013),
            &json_ops::INSTANCE,
            is_error
        );
    }
}
