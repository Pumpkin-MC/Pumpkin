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
