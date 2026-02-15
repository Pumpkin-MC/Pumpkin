use crate::serialization::HasValue;
use crate::serialization::codec::Codec;
use crate::serialization::coders::{Decoder, Encoder};
use crate::serialization::data_result::DataResult;
use crate::serialization::dynamic_ops::DynamicOps;
use crate::serialization::lifecycle::Lifecycle;
use crate::serialization::map_codecs::base::BaseMapCodec;
use crate::serialization::struct_builder::StructBuilder;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;

/// A type of [`Codec`] for a map with no known list of keys.
pub struct UnboundedMapCodec<K: Codec + 'static, V: Codec + 'static>
where
    K::Value: Display + Eq + Hash,
{
    pub(crate) key_codec: &'static K,
    pub(crate) element_codec: &'static V,
}

impl<K: Codec, V: Codec> BaseMapCodec for UnboundedMapCodec<K, V>
where
    <K as HasValue>::Value: Display + Eq + Hash,
{
    type Key = K::Value;
    type KeyCodec = K;
    type Element = V::Value;
    type ElementCodec = V;

    fn key_codec(&self) -> &'static Self::KeyCodec {
        self.key_codec
    }

    fn element_codec(&self) -> &'static Self::ElementCodec {
        self.element_codec
    }
}

impl<K: Codec, V: Codec> HasValue for UnboundedMapCodec<K, V>
where
    <K as HasValue>::Value: Display + Eq + Hash,
{
    type Value = HashMap<K::Value, V::Value>;
}

impl<K: Codec, V: Codec> Encoder for UnboundedMapCodec<K, V>
where
    <K as HasValue>::Value: Display + Eq + Hash,
{
    fn encode<T: Display + PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T> {
        BaseMapCodec::encode(self, input, ops, ops.map_builder()).build(prefix)
    }
}

impl<K: Codec, V: Codec> Decoder for UnboundedMapCodec<K, V>
where
    <K as HasValue>::Value: Display + Eq + Hash,
{
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(Self::Value, T)> {
        ops.get_map(&input)
            .with_lifecycle(Lifecycle::Stable)
            .flat_map(|map| BaseMapCodec::decode(self, &map, ops))
            .map(|r| (r, input))
    }
}

#[cfg(test)]
mod test {
    use crate::assert_decode;
    use crate::serialization::codec::*;
    use crate::serialization::codecs::primitive::{BoolCodec, IntCodec, StringCodec};
    use crate::serialization::codecs::unbounded_map::UnboundedMapCodec;
    use crate::serialization::codecs::validated::ValidatedCodec;
    use crate::serialization::coders::Decoder;
    use crate::serialization::coders::Encoder;
    use crate::serialization::json_ops;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn simple_encoding() {
        pub static SCORES_CODEC: UnboundedMapCodec<StringCodec, IntCodec> =
            unbounded_map(&STRING_CODEC, &INT_CODEC);

        let mut map = HashMap::<String, i32>::new();

        map.insert("Amy".to_string(), 10);
        map.insert("Leo".to_string(), 24);
        map.insert("Patrick".to_string(), -65);

        assert_eq!(
            SCORES_CODEC
                .encode_start(&map, &json_ops::INSTANCE)
                .expect("Encoding scores failed"),
            json!({"Amy": 10, "Leo": 24, "Patrick": -65})
        );
    }

    #[test]
    fn number_key_encoding() {
        // A basic implementation to check if a number is prime.
        fn is_prime(number: u32) -> bool {
            if number < 2 {
                return false;
            }
            for i in 2..number {
                if number.is_multiple_of(i) {
                    return false;
                }
            }
            true
        }

        // A codec to store whether a number is prime or not.
        // We use a transformer to keep the keys in a string form even while working with `u32` keys.
        pub static PRIME_MAP_CODEC: UnboundedMapCodec<XmapCodec<u32, StringCodec>, BoolCodec> =
            unbounded_map(
                &xmap(
                    &STRING_CODEC,
                    |s| s.parse().expect("Could not parse String"),
                    |u: &u32| u.to_string(),
                ),
                &BOOL_CODEC,
            );

        let mut map = HashMap::<u32, bool>::new();

        // Calculate the map for the first 20 numbers.
        for i in 1..=20 {
            map.insert(i, is_prime(i));
        }

        assert_eq!(
            PRIME_MAP_CODEC
                .encode_start(&map, &json_ops::INSTANCE)
                .expect("Encoding prime map failed"),
            json!({
                "1": false, "2": true, "3": true, "4": false, "5": true, "6": false, "7": true, "8": false, "9": false, "10": false,
                "11": true, "12": false, "13": true, "14": false, "15": false, "16": false, "17": true, "18": false, "19": true, "20": false
            })
        );
    }

    #[test]
    fn decoding() {
        // A codec storing a frequency for each letter.
        // Each key must only be 1 character long (to make it a letter).
        // There must be at least 1 key.
        pub static LETTER_FREQUENCY_CODEC: ValidatedCodec<
            UnboundedMapCodec<ValidatedCodec<StringCodec>, UnsignedLongCodec>,
        > = validate(
            &unbounded_map(
                &validate(&STRING_CODEC, |s| {
                    if s.len() == 1 {
                        Ok(())
                    } else {
                        Err("String must be exactly 1 character long".to_string())
                    }
                }),
                &UNSIGNED_LONG_CODEC,
            ),
            |m| {
                if m.is_empty() {
                    Err("Map must not be empty".to_string())
                } else {
                    Ok(())
                }
            },
        );

        assert_decode!(
            LETTER_FREQUENCY_CODEC,
            json!({"a": 13, "c": 34, "x": 1, "e": 21}),
            &json_ops::INSTANCE,
            is_success
        );

        assert_decode!(
            LETTER_FREQUENCY_CODEC,
            json!({"b": 45, "w": 10, "l": 90, "word": 5}),
            &json_ops::INSTANCE,
            is_error
        );

        assert_decode!(
            LETTER_FREQUENCY_CODEC,
            json!({}),
            &json_ops::INSTANCE,
            is_error
        );
    }
}
