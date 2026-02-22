use crate::codec::Codec;
use crate::coders::{Decoder, Encoder};
use crate::data_result::DataResult;
use crate::dynamic_ops::DynamicOps;
use crate::HasValue;
use crate::util::either::Either;
use std::fmt::Display;

/// A codec that can serialize/deserialize one of two types, with a codec for each one.
///
/// This evaluates the left codec first, and if the [`DataResult`] for it is invalid,
/// it evaluates the right codec.
pub struct EitherCodec<L: Codec + 'static, R: Codec + 'static> {
    left_codec: &'static L,
    right_codec: &'static R,
}

impl<L: Codec, R: Codec> HasValue for EitherCodec<L, R> {
    type Value = Either<L::Value, R::Value>;
}

impl<L: Codec, R: Codec> Encoder for EitherCodec<L, R> {
    fn encode<T: Display + PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T> {
        match &input {
            Either::Left(l) => self.left_codec.encode(l, ops, prefix),
            Either::Right(r) => self.right_codec.encode(r, ops, prefix),
        }
    }
}

impl<L: Codec, R: Codec> Decoder for EitherCodec<L, R> {
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(Self::Value, T)> {
        let left = self
            .left_codec
            .decode(input.clone(), ops)
            .map(|(l, t)| (Either::Left(l), t));

        // If the left result is a success, return that.
        if left.is_success() {
            return left;
        }

        let right = self
            .right_codec
            .decode(input, ops)
            .map(|(r, t)| (Either::Right(r), t));

        // If the right result is a success, return that.
        if right.is_success() {
            return right;
        }

        // Since no result is a complete success by this point, we look for partial results.

        if left.has_result_or_partial() {
            return left;
        }

        if right.has_result_or_partial() {
            return right;
        }

        DataResult::error(format!(
            "Failed to parse either. First: {}; Second: {}",
            left.get_message().unwrap(),
            right.get_message().unwrap()
        ))
    }
}

/// Creates a new `EitherCodec` with the provided left and right codecs for serializing/deserializing both possible types.
pub(crate) const fn new_either_codec<L: Codec, R: Codec>(
    left_codec: &'static L,
    right_codec: &'static R,
) -> EitherCodec<L, R> {
    EitherCodec {
        left_codec,
        right_codec,
    }
}

#[cfg(test)]
mod test {
    use crate::codec::{
        DOUBLE_CODEC, FieldMapCodec, INT_CODEC, STRING_CODEC, either, field, unbounded_map,
    };
    use crate::codecs::either::EitherCodec;
    use crate::codecs::primitive::{DoubleCodec, IntCodec, StringCodec};
    use crate::codecs::unbounded_map::UnboundedMapCodec;
    use crate::coders::{Decoder, Encoder};
    use crate::json_ops;
    use crate::map_codec::for_getter;
    use crate::struct_codecs::StructCodec2;
    use crate::struct_codec;
    use crate::util::either::Either;
    use serde_json::json;

    #[test]
    fn simple() {
        pub static EITHER_INT_STRING_CODEC: EitherCodec<IntCodec, StringCodec> =
            either(&INT_CODEC, &STRING_CODEC);

        // Encoding

        assert_eq!(
            EITHER_INT_STRING_CODEC
                .encode_start(&Either::Left(5), &json_ops::INSTANCE)
                .expect("Encoding should succeed"),
            json!(5)
        );

        assert_eq!(
            EITHER_INT_STRING_CODEC
                .encode_start(
                    &Either::Right("I am some text.".to_string()),
                    &json_ops::INSTANCE
                )
                .expect("Encoding should succeed"),
            json!("I am some text.")
        );

        // Decoding

        assert_eq!(
            EITHER_INT_STRING_CODEC
                .parse(json!(-238), &json_ops::INSTANCE)
                .expect("Decoding should succeed"),
            Either::Left(-238)
        );
        assert_eq!(
            EITHER_INT_STRING_CODEC
                .parse(json!("hello"), &json_ops::INSTANCE)
                .expect("Decoding should succeed"),
            Either::Right("hello".to_string())
        );
        assert!(
            EITHER_INT_STRING_CODEC
                .parse(json!(true), &json_ops::INSTANCE)
                .get_message()
                .expect("Decoding should fail")
                .starts_with("Failed to parse either.")
        );
    }

    // A situation where two codecs could possibly decode valid but different values.
    // This test only checks for decoding.
    #[test]
    fn intersecting_codecs() {
        /// A type to store a complex number (a number with both a real and imaginary part).
        #[derive(Debug, PartialEq, Clone)]
        struct ComplexNumber(f64, f64);

        pub type ComplexNumberCodec =
            StructCodec2<ComplexNumber, FieldMapCodec<DoubleCodec>, FieldMapCodec<DoubleCodec>>;
        pub static COMPLEX_NUMBER_CODEC: ComplexNumberCodec = struct_codec!(
            for_getter(field(&DoubleCodec, "real"), |n| &n.0),
            for_getter(field(&DoubleCodec, "imaginary"), |n| &n.1),
            ComplexNumber
        );

        pub type DoubleMapCodec = UnboundedMapCodec<StringCodec, DoubleCodec>;
        pub static DOUBLE_MAP_CODEC: DoubleMapCodec = unbounded_map(&STRING_CODEC, &DOUBLE_CODEC);

        pub static COMPLEX_NUMBER_FIRST_EITHER_CODEC: EitherCodec<
            ComplexNumberCodec,
            DoubleMapCodec,
        > = either(&COMPLEX_NUMBER_CODEC, &DOUBLE_MAP_CODEC);
        pub static DOUBLE_MAP_FIRST_EITHER_CODEC: EitherCodec<DoubleMapCodec, ComplexNumberCodec> =
            either(&DOUBLE_MAP_CODEC, &COMPLEX_NUMBER_CODEC);

        // We expect a complex number first, as that is the first to be checked.
        assert_eq!(
            COMPLEX_NUMBER_FIRST_EITHER_CODEC
                .parse(
                    json!(
                        {
                            "real": 10.1,
                            "imaginary": -4.5
                        }
                    ),
                    &json_ops::INSTANCE
                )
                .expect("Decoding should succeed")
                .expect_left("Expected complex number"),
            ComplexNumber(10.1, -4.5)
        );

        // This should be a map, as the complex number codec fails due to a missing field.
        assert_eq!(
            COMPLEX_NUMBER_FIRST_EITHER_CODEC
                .parse(
                    json!(
                        {
                            "real": 10.1,
                            "second": -4.5,
                            "third": 1
                        }
                    ),
                    &json_ops::INSTANCE
                )
                .expect("Decoding should succeed")
                .expect_right("Expected double map")
                .len(),
            3
        );

        assert_eq!(
            DOUBLE_MAP_FIRST_EITHER_CODEC
                .parse(
                    json!(
                        {
                            "a": 1,
                            "b": 2,
                            "c": 3,
                            "d": 4
                        }
                    ),
                    &json_ops::INSTANCE
                )
                .expect("Decoding should succeed")
                .expect_left("Expected double map")
                .len(),
            4
        );
    }
}
