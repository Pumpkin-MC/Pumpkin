use crate::map_like::MapLike;
use crate::struct_builder::StructBuilder;
use crate::{DataResult, Decode, DynamicOps, Encode, Lifecycle};
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::{BuildHasher, Hash};

fn encode_base_hash_map<
    O: DynamicOps,
    K: Encode + Eq + Hash + Display,
    V: Encode,
    S: BuildHasher + Default,
>(
    map: &HashMap<K, V, S>,
    ops: &'static O,
    mut prefix: impl StructBuilder<Value = O::Value>,
) -> impl StructBuilder<Value = O::Value> {
    for (key, element) in map {
        prefix =
            prefix.add_key_result_value_result(key.encode_start(ops), element.encode_start(ops));
    }
    prefix
}

fn decode_base_hash_map<
    O: DynamicOps,
    K: Decode + Eq + Hash + Display,
    V: Decode,
    S: BuildHasher + Default,
>(
    input: &impl MapLike<Value = O::Value>,
    ops: &'static O,
) -> DataResult<HashMap<K, V, S>> {
    let mut read_map: HashMap<K, V, S> = HashMap::with_hasher(S::default());
    let mut failed: Vec<(O::Value, O::Value)> = vec![];

    let result = input.iter().fold(
        DataResult::new_success_with_lifecycle((), Lifecycle::Stable),
        |r, (k, e)| {
            // First, we try to parse the key and value.
            let key_result = K::parse(k.clone(), ops);
            let element_result = V::parse(e.clone(), ops);

            let entry_result =
                key_result.apply_2_and_make_stable(|kr, er| (kr, er), element_result);
            let accumulated = r.add_message(&entry_result);
            let entry = entry_result.into_result_or_partial();

            if let Some((key, element)) = entry {
                // If this parses successfully, we try adding it to our map.
                if read_map.contains_key(&key) {
                    // There was already a value for this key.
                    failed.push((k, e.clone()));
                    return accumulated.add_message::<()>(&DataResult::new_error(format!(
                        "Duplicate entry for key: {key}"
                    )));
                }
                read_map.insert(key, element);
            } else {
                // Could not parse.
                failed.push((k, e.clone()));
            }

            accumulated
        },
    );

    let errors = ops.create_map(failed);

    result
        .with_complete_or_partial(read_map)
        .map_error(|e| format!("{e} (Missed inputs: {errors})"))
}

impl<K, V, S: BuildHasher + Default> Encode for HashMap<K, V, S>
where
    K: Encode + Eq + Hash + Display,
    V: Encode,
{
    fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
        encode_base_hash_map::<O, K, V, S>(self, ops, ops.map_builder()).build(prefix)
    }
}

impl<K, V, S: BuildHasher + Default> Decode for HashMap<K, V, S>
where
    K: Decode + Eq + Hash + Display,
    V: Decode,
{
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input)
            .with_lifecycle(Lifecycle::Stable)
            .flat_map(|map| decode_base_hash_map(&map, ops))
            .map(|r| (r, input))
    }
}

#[cfg(test)]
mod test {
    use crate::codec::*;
    use crate::json_ops::JsonOps;
    use crate::{
        FlatTryFrom, assert_decode, assert_encode_success, comap_flat_map_codec_impl,
        flat_xmap_codec_impl,
    };
    use serde_json::json;
    use std::collections::HashMap;
    use std::fmt::{Display, Formatter};

    #[test]
    fn simple_encoding() {
        let mut map = HashMap::<String, i32>::new();

        map.insert("Amy".to_string(), 10);
        map.insert("Leo".to_string(), 24);
        map.insert("Patrick".to_string(), -65);

        assert_encode_success!(map, JsonOps, json!({"Amy": 10, "Leo": 24, "Patrick": -65}));
    }

    #[test]
    fn string_integer_map() {
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

        /// A `u32` wrapper that encodes a `String`.
        #[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Eq, Hash)]
        struct StringInteger(u32);

        impl Display for StringInteger {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<&StringInteger> for String {
            fn from(value: &StringInteger) -> Self {
                // This will always succeed.
                value.0.to_string()
            }
        }

        impl FlatTryFrom<String> for StringInteger {
            fn flat_try_from(value: String) -> DataResult<Self> {
                // Try to parse an integer.
                value.parse().map_or_else(
                    |_| DataResult::new_error("Could not parse String"),
                    |i| DataResult::new_success(Self(i)),
                )
            }
        }

        comap_flat_map_codec_impl!(String => StringInteger, StringInteger::flat_try_from, String::from);

        let mut map = HashMap::<StringInteger, bool>::new();

        // Calculate the map for the first 20 numbers.
        for i in 1..=20 {
            map.insert(StringInteger(i), is_prime(i));
        }

        assert_encode_success!(
            map,
            JsonOps,
            json!({
                "1": false, "2": true, "3": true, "4": false, "5": true, "6": false, "7": true, "8": false, "9": false, "10": false,
                "11": true, "12": false, "13": true, "14": false, "15": false, "16": false, "17": true, "18": false, "19": true, "20": false
            })
        );

        assert_decode!(
            HashMap<StringInteger, bool>,
            json!({
                "1": true, "2": true, "3": true, "4": false, "5": false
            }),
            JsonOps,
            is_success
        );
    }

    #[test]
    fn letter_frequency() {
        /// A wrapper of a `String` that only allows a single letter.
        #[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Hash)]
        struct Letter(String);

        impl Display for Letter {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl FlatTryFrom<String> for Letter {
            fn flat_try_from(value: String) -> DataResult<Self> {
                // Try to parse a single letter.
                if value.len() == 1 {
                    DataResult::new_success(Self(value))
                } else {
                    DataResult::new_error(format!("Not a letter: {value}"))
                }
            }
        }

        impl FlatTryFrom<&Letter> for String {
            fn flat_try_from(value: &Letter) -> DataResult<Self> {
                Letter::flat_try_from(value.0.clone()).map(|l| l.0)
            }
        }

        flat_xmap_codec_impl!(String => Letter, Letter::flat_try_from, String::flat_try_from);

        type LetterData = HashMap<Letter, u64>;

        let mut map = LetterData::new();
        map.insert(Letter("b".to_string()), 62);
        map.insert(Letter("z".to_string()), 2342);

        assert_encode_success!(map, JsonOps, json!({"b": 62, "z": 2342}));

        let mut map = LetterData::new();
        map.insert(Letter("d".to_string()), 12452);
        map.insert(Letter("candy".to_string()), 2342);

        assert!(map.encode_start(&JsonOps).is_error());

        assert_decode!(
            LetterData,
            json!({"a": 13, "c": 34, "x": 1, "e": 21}),
            JsonOps,
            is_success
        );

        assert_decode!(
            LetterData,
            json!({"b": 45, "w": 10, "l": 90, "word": 5}),
            JsonOps,
            is_error
        );
    }
}
