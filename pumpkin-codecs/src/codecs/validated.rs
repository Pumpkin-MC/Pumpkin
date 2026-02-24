use crate::HasValue;
use crate::codec::Codec;
use crate::coders::{Decoder, Encoder};
use crate::data_result::DataResult;
use crate::dynamic_ops::DynamicOps;
use std::fmt::Display;

/// A validator codec that validates any values before encoding and after decoding.
pub struct ValidatedCodec<C: Codec + 'static> {
    codec: &'static C,
    /// The validator function used.
    validator: fn(&C::Value) -> Result<(), String>,
}

impl<C: Codec> HasValue for ValidatedCodec<C> {
    type Value = C::Value;
}

impl<C: Codec> Encoder for ValidatedCodec<C> {
    fn encode<T: Display + PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T> {
        (self.validator)(input).map_or_else(
            |error| DataResult::error(error),
            |()| self.codec.encode(input, ops, prefix),
        )
    }
}

impl<C: Codec> Decoder for ValidatedCodec<C> {
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(Self::Value, T)> {
        self.codec.decode(input, ops).flat_map(|decoded| {
            (self.validator)(&decoded.0)
                .map_or_else(DataResult::error, |()| DataResult::success(decoded))
        })
    }
}

/// Creates a new [`ValidatedCodec`].
pub(crate) const fn new_validated_codec<C: Codec>(
    codec: &'static C,
    validator: fn(&C::Value) -> Result<(), String>,
) -> ValidatedCodec<C> {
    ValidatedCodec { codec, validator }
}

#[cfg(test)]
mod test {
    use crate::assert_decode;
    use crate::codec::*;
    use crate::codecs::primitive::{IntCodec, StringCodec};
    use crate::codecs::validated::ValidatedCodec;
    use crate::coders::Decoder;
    use crate::coders::Encoder;
    use crate::json_ops;
    use serde_json::json;

    #[test]
    fn even_int_validation() {
        // An `int` codec that only accepts even numbers.
        pub static EVEN_INT_CODEC: ValidatedCodec<IntCodec> = validate(&INT_CODEC, |value| {
            if value % 2 == 0 {
                Ok(())
            } else {
                Err(String::from("Not an even number"))
            }
        });

        assert_eq!(
            EVEN_INT_CODEC
                .encode_start(&2, &json_ops::INSTANCE)
                .expect("Encoding panicked"),
            json!(2)
        );
        assert_eq!(
            EVEN_INT_CODEC
                .encode_start(&-56, &json_ops::INSTANCE)
                .expect("Encoding panicked"),
            json!(-56)
        );
        assert!(
            EVEN_INT_CODEC
                .encode_start(&-135, &json_ops::INSTANCE)
                .is_error()
        );

        assert_decode!(EVEN_INT_CODEC, json!(0), &json_ops::INSTANCE, is_success);
        assert_decode!(EVEN_INT_CODEC, json!(3456), &json_ops::INSTANCE, is_success);
        assert_decode!(EVEN_INT_CODEC, json!(-12345), &json_ops::INSTANCE, is_error);
        assert_decode!(EVEN_INT_CODEC, json!(153453), &json_ops::INSTANCE, is_error);
    }

    #[test]
    fn player_name_validation() {
        // A codec of a Minecraft player name, which has the following rules:
        // - The length must be between 3-16 characters long.
        // - They must only have alphanumeric characters and underscores.
        pub static PLAYER_NAME_CODEC: ValidatedCodec<StringCodec> = validate(&STRING_CODEC, |s| {
            if !(3..=16).contains(&s.len()) {
                return Err(String::from(
                    "Player name must be between 3-16 characters long (inclusive)",
                ));
            }
            if !s.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(String::from(
                    "Player name must only contain alphanumeric characters and underscores",
                ));
            }
            Ok(())
        });

        assert!(
            PLAYER_NAME_CODEC
                .encode_start(&String::from("Player"), &json_ops::INSTANCE)
                .is_success()
        );
        assert!(
            PLAYER_NAME_CODEC
                .encode_start(&String::from("abcd1234"), &json_ops::INSTANCE)
                .is_success()
        );
        assert!(
            PLAYER_NAME_CODEC
                .encode_start(&String::from("has some spaces"), &json_ops::INSTANCE)
                .is_error()
        );
        assert!(
            PLAYER_NAME_CODEC
                .encode_start(&String::from("XxXxVeryLongNamexXxX"), &json_ops::INSTANCE)
                .is_error()
        );
        assert!(
            PLAYER_NAME_CODEC
                .encode_start(&String::from("ILovePizza$"), &json_ops::INSTANCE)
                .is_error()
        );

        assert_decode!(
            PLAYER_NAME_CODEC,
            json!("Pumpkin"),
            &json_ops::INSTANCE,
            is_success
        );
        assert_decode!(
            PLAYER_NAME_CODEC,
            json!("IGoByNoNames__"),
            &json_ops::INSTANCE,
            is_success
        );
        assert_decode!(
            PLAYER_NAME_CODEC,
            json!("#idk"),
            &json_ops::INSTANCE,
            is_error
        );
    }
}
