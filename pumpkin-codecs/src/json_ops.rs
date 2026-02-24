use std::fmt::{Debug, Display};

use crate::{Number, data_result::DataResult, dynamic_ops::DynamicOps, map_like::MapLike};

use crate::lifecycle::Lifecycle;
use crate::struct_builder::{ResultStructBuilder, StringStructBuilder, StructBuilder};
use crate::{impl_string_struct_builder, impl_struct_builder};
use serde_json::{Map, Value};

/// A [`DynamicOps`] to serialize to/deserialize from JSON data.
pub struct JsonOps {
    compressed: bool,
}

/// A normal instance of [`JsonOps`], which serializes/deserializes normal JSON data.
pub static INSTANCE: JsonOps = JsonOps { compressed: false };

/// A normal instance of [`JsonOps`], which serializes/deserializes compressed JSON data.
///
/// *Compressed* JSON data is a little more lenient with placing values at places that expect something else.
/// This allows JSON to be compressed to a single string.
pub static COMPRESSED: JsonOps = JsonOps { compressed: true };

impl JsonOps {
    /// A function to get a JSON value as a string, similar to Google's GSON's `getAsString()` method for `JsonElement`.
    /// This is to keep parity with the `JsonOps` methods that check for `compressed`.
    ///
    /// In particular, this method may return `Some` for *ONLY* the following:
    /// - Booleans (always)
    /// - Numbers (always)
    /// - Strings (always)
    /// - Arrays with exactly 1 element (in this case, this is called for that element).
    ///
    /// Any other case returns `None`.
    fn get_as_string(input: &Value) -> Option<String> {
        match input {
            Value::Array(elements) => {
                // If we have an array, it must only have 1 element.
                if elements.len() == 1 {
                    Self::get_as_string(&elements[0])
                } else {
                    None
                }
            }
            Value::Bool(b) => Some(b.to_string()),
            Value::Number(n) => Some(n.to_string()),
            Value::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    /// Whether a JSON value is considered to be a valid key.
    ///
    /// If this returns `true`, it is safe to say that calling [`get_as_string`] with `input` will always return a [`Some`].
    const fn is_valid_key(&self, input: &Value) -> bool {
        // Normal mode: has to be a string.
        // Compressed mode: can be any JSON primitive.
        if self.compressed {
            matches!(input, Value::String(_) | Value::Number(_) | Value::Bool(_))
        } else {
            matches!(input, Value::String(_))
        }
    }
}

impl DynamicOps for JsonOps {
    type Value = Value;
    type StructBuilder = JsonStructBuilder;

    fn empty(&self) -> Self::Value {
        Value::Null
    }

    fn create_number(&self, n: Number) -> Self::Value {
        n.into()
    }

    fn create_bool(&self, data: bool) -> Self::Value {
        Value::Bool(data)
    }

    fn create_string(&self, data: &str) -> Self::Value {
        Value::String(data.to_owned())
    }

    fn create_list<I>(&self, values: I) -> Self::Value
    where
        I: IntoIterator<Item = Self::Value>,
    {
        Value::Array(values.into_iter().collect())
    }

    fn create_map<I>(&self, entries: I) -> Self::Value
    where
        I: IntoIterator<Item = (Self::Value, Self::Value)>,
    {
        Value::Object(
            entries
                .into_iter()
                .filter_map(|(k, v)| Self::get_as_string(&k).map(|k| (k, v)))
                .collect(),
        )
    }

    fn get_bool(&self, input: &Self::Value) -> DataResult<bool> {
        if let Value::Bool(b) = input {
            DataResult::success(*b)
        } else {
            DataResult::error(format!("Not a boolean: {input}"))
        }
    }

    fn get_number(&self, input: &Self::Value) -> DataResult<Number> {
        match input {
            Value::Number(_) => {
                return input.try_into().map_or_else(
                    |_| DataResult::error(format!("Not a number: {input}")),
                    DataResult::success,
                );
            }
            Value::String(string) => {
                if self.compressed
                    && let Ok(r) = string.parse::<i32>()
                {
                    return DataResult::success(Number::Int(r));
                }
            }
            _ => {}
        }
        DataResult::error(format!("Not a number: {input}"))
    }

    fn get_string(&self, input: &Self::Value) -> DataResult<String> {
        if matches!(input, Value::String(_))
            || (matches!(input, Value::Number(_)) && self.compressed)
        {
            // Unwrapping is fine as only strings and numbers are possible here.
            DataResult::success(Self::get_as_string(input).unwrap())
        } else {
            DataResult::error(format!("Not a string: {input}"))
        }
    }

    fn get_map_iter<'a>(
        &self,
        input: &'a Self::Value,
    ) -> DataResult<impl Iterator<Item = (Self::Value, &'a Self::Value)> + 'a> {
        if let Value::Object(map) = input {
            DataResult::success(map.iter().map(|(k, v)| (Value::String(k.clone()), v)))
        } else {
            DataResult::error(format!("Not a JSON object: {input}"))
        }
    }

    fn get_map<'a>(
        &self,
        input: &'a Self::Value,
    ) -> DataResult<impl MapLike<Value = Self::Value> + 'a> {
        if let Value::Object(map) = input {
            DataResult::success(JsonMapLike { map })
        } else {
            DataResult::error(format!("Not a JSON object: {input}"))
        }
    }

    fn get_iter(&self, input: Self::Value) -> DataResult<impl Iterator<Item = Self::Value>> {
        // This only works for JSON arrays.
        if let Value::Array(list) = input {
            DataResult::success(list.into_iter())
        } else {
            DataResult::error(format!("Not a JSON array: {input}"))
        }
    }

    fn merge_into_list(&self, list: Self::Value, value: Self::Value) -> DataResult<Self::Value> {
        if matches!(list, Value::Array(_)) || list == self.empty() {
            let mut result_vec = vec![];
            if let Value::Array(a) = list {
                result_vec.extend(a);
            }

            result_vec.push(value);

            DataResult::success(Value::Array(result_vec))
        } else {
            DataResult::partial_error(format!("Not a list: {list}"), list)
        }
    }

    fn merge_values_into_list<I>(&self, list: Self::Value, values: I) -> DataResult<Self::Value>
    where
        I: IntoIterator<Item = Self::Value>,
    {
        if matches!(list, Value::Array(_)) || list == self.empty() {
            let mut result_vec = vec![];
            if let Value::Array(a) = list {
                result_vec.extend(a);
            }

            result_vec.extend(values);

            DataResult::success(Value::Array(result_vec))
        } else {
            DataResult::partial_error(format!("Not a list: {list}"), list)
        }
    }

    fn merge_into_map(
        &self,
        map: Self::Value,
        key: Self::Value,
        value: Self::Value,
    ) -> DataResult<Self::Value>
    where
        Self::Value: Clone,
    {
        if !matches!(map, Value::Object(_)) && map != self.empty() {
            return DataResult::partial_error(format!("Not a map: {map}"), map);
        }

        if !self.is_valid_key(&key) {
            return DataResult::partial_error(format!("Key is not a string: {key}"), map);
        }

        let mut output_map = Map::new();

        if let Value::Object(mut m) = map {
            output_map.append(&mut m);
        }
        output_map.insert(Self::get_as_string(&key).unwrap(), value);

        DataResult::success(Value::Object(output_map))
    }

    fn merge_map_like_into_map<M>(
        &self,
        map: Self::Value,
        other_map_like: M,
    ) -> DataResult<Self::Value>
    where
        M: MapLike<Value = Self::Value>,
        Self::Value: Clone,
    {
        if matches!(map, Value::Object(_)) || map == self.empty() {
            let mut output_map = Map::new();

            if let Value::Object(mut m) = map {
                output_map.append(&mut m);
            }

            // Store the missed entries.
            let mut missed = vec![];

            for entry in other_map_like.iter() {
                if self.is_valid_key(&entry.0) {
                    output_map.insert(Self::get_as_string(&entry.0).unwrap(), entry.1.clone());
                } else {
                    missed.push(entry.0);
                }
            }

            let object = Value::Object(output_map);
            let pretty_missed = serde_json::to_string_pretty(&missed);
            if missed.is_empty() {
                DataResult::success(object)
            } else {
                DataResult::partial_error(
                    format!(
                        "Some keys are not strings{}",
                        pretty_missed.map_or_else(|_| String::new(), |r| format!(": {r}"))
                    ),
                    object,
                )
            }
        } else {
            DataResult::partial_error(format!("Not a map: {map}"), map)
        }
    }

    fn remove(&self, input: Self::Value, key: &str) -> Value {
        if let Value::Object(m) = input {
            Value::Object(m.into_iter().filter(|(k, _)| k != key).collect())
        } else {
            input
        }
    }

    fn compress_maps(&self) -> bool {
        self.compressed
    }

    fn convert_to<U>(&self, out_ops: &impl DynamicOps<Value = U>, input: Self::Value) -> U {
        match input {
            Value::Null => out_ops.empty(),
            Value::Bool(b) => out_ops.create_bool(b),
            Value::String(s) => out_ops.create_string(&s),
            Value::Array(_) => self.convert_list(out_ops, input),
            Value::Object(_) => self.convert_map(out_ops, input),

            Value::Number(n) => {
                // First, check for possible integers.
                if let Some(l) = n.as_i64() {
                    if (l as i8) as i64 == l {
                        return out_ops.create_byte(l as i8);
                    } else if (l as i16) as i64 == l {
                        return out_ops.create_short(l as i16);
                    } else if (l as i32) as i64 == l {
                        return out_ops.create_int(l as i32);
                    }
                    out_ops.create_long(l)
                // If no integer is possible, check for possible floating-point values.
                } else if let Some(f) = n.as_f64() {
                    if (f as f32) as f64 == f {
                        return out_ops.create_float(f as f32);
                    }
                    out_ops.create_double(f)
                } else {
                    // Just in case.
                    out_ops.create_double(0.0)
                }
            }
        }
    }

    fn map_builder(&'static self) -> Self::StructBuilder {
        JsonStructBuilder {
            builder: DataResult::success_with_lifecycle(
                Value::Object(Map::new()),
                Lifecycle::Stable,
            ),
        }
    }
}

/// An implementation of [`MapLike`] for JSON objects.
/// The lifetime is that of the referenced map.
struct JsonMapLike<'a> {
    map: &'a Map<String, Value>,
}

impl MapLike for JsonMapLike<'_> {
    type Value = Value;

    fn get(&self, key: &Self::Value) -> Option<&Self::Value> {
        JsonOps::get_as_string(key).and_then(|s| self.get_str(&s))
    }

    fn get_str(&self, key: &str) -> Option<&Self::Value> {
        self.map.get(key)
    }

    fn iter(&self) -> impl Iterator<Item = (Self::Value, &Self::Value)> + '_ {
        self.map.iter().map(|(k, v)| (Value::String(k.clone()), v))
    }
}

impl Display for JsonMapLike<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.map.fmt(f)
    }
}

/// An implementation of [`StructBuilder`] for JSON objects.
pub struct JsonStructBuilder {
    builder: DataResult<Value>,
}

impl ResultStructBuilder for JsonStructBuilder {
    type Result = Value;

    fn build_with_builder(
        self,
        builder: Self::Result,
        prefix: Self::Value,
    ) -> DataResult<Self::Value> {
        match prefix {
            Value::Null => DataResult::success(builder),
            Value::Object(mut map) => {
                match builder {
                    Value::Object(builder_map) => {
                        for (k, v) in builder_map {
                            map.insert(k.clone(), v.clone());
                        }
                    }
                    _ => unreachable!(),
                }
                DataResult::success(Value::Object(map))
            }
            _ => DataResult::partial_error(format!("Prefix is not a map: {prefix}"), prefix),
        }
    }
}

impl StructBuilder for JsonStructBuilder {
    type Value = Value;

    impl_struct_builder!(builder);
    impl_string_struct_builder!(builder, INSTANCE);
}

impl StringStructBuilder for JsonStructBuilder {
    fn append(&self, key: &str, value: Self::Value, mut builder: Self::Result) -> Self::Result {
        builder
            .as_object_mut()
            .unwrap()
            .insert(key.to_string(), value);
        builder
    }
}
