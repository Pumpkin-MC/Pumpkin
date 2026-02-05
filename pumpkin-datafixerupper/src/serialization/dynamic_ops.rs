use std::{collections::HashMap, fmt::Display};

use crate::serialization::{Number, data_result::DataResult, map_like::MapLike};

/// Generates default implementations for numeric creators.
macro_rules! create_numbers_impl {
    (
        $(
            $(#[$meta:meta])*
            fn $name:ident($arg:ident : $ty:ty) => $variant:ident
        ),* $(,)?
    ) => {
        $(
            $(#[$meta])*
            fn $name(&self, $arg: $ty) -> Self::Value {
                self.create_number(Number::$variant($arg))
            }
        )*
    };
}

/// A trait describing methods to read and write a specific format (like NBT or JSON).
/// The [`Value`] of this trait is the type that can be used to represent anything in this format.
pub trait DynamicOps {
    type Value;

    /// Returns how an empty value is represented by this `DynamicOps`.
    fn empty(&self) -> Self::Value;

    /// Returns how an empty list is represented by this `DynamicOps`.
    fn empty_list(&self) -> Self::Value {
        self.create_list(vec![])
    }

    /// Returns how an empty map is represented by this `DynamicOps`.
    fn empty_map(&self) -> Self::Value {
        self.create_map(HashMap::new())
    }

    /// Returns how a generic number is represented by this `DynamicOps`.
    fn create_number(&self, n: Number) -> Self::Value;

    create_numbers_impl! {
        /// Returns how a generic `byte` in Java (equivalent to [`i8`]) is represented by this `DynamicOps`.
        fn create_byte(data: i8) => Byte,

        /// Returns how a generic `short` in Java (equivalent to [`i16`]) is represented by this `DynamicOps`.
        fn create_short(data: i16) => Short,

        /// Returns how a generic `int` in Java (equivalent to [`i32`]) is represented by this `DynamicOps`.
        fn create_int(data: i32) => Int,

        /// Returns how a generic `long` in Java (equivalent to [`i64`]) is represented by this `DynamicOps`.
        fn create_long(data: i64) => Long,

        /// Returns how a generic `float` in Java (equivalent to [`f32`]) is represented by this `DynamicOps`.
        fn create_float(data: f32) => Float,

        /// Returns how a generic `double` in Java (equivalent to [`f64`]) is represented by this `DynamicOps`.
        fn create_double(data: f64) => Double,
    }

    /// Returns how a boolean is represented by this `DynamicOps`.
    fn create_bool(&self, data: bool) -> Self::Value {
        self.create_byte(i8::from(data))
    }

    /// Returns how a string is represented by this `DynamicOps`.
    fn create_string(&self, data: String) -> Self::Value;

    /// Returns how a list is represented by this `DynamicOps`.
    fn create_list<I>(&self, values: I) -> Self::Value
    where
        I: IntoIterator<Item = Self::Value>;

    /// Returns how a map is represented by this `DynamicOps`.
    fn create_map<I>(&self, entries: I) -> Self::Value
    where
        I: IntoIterator<Item = (Self::Value, Self::Value)>;

    /// Tries to get a number represented by this `DynamicOps`.
    fn get_number(&self, input: &Self::Value) -> DataResult<Number>;

    /// Tries to get a string represented by this `DynamicOps`.
    fn get_string(&self, input: &Self::Value) -> DataResult<String>;

    /// Gets an [`Iterator`] from a map represented by this `DynamicOps`.
    fn get_map_iter<'a>(
        &'a self,
        input: &'a Self::Value,
    ) -> DataResult<impl Iterator<Item = (&'a Self::Value, &'a Self::Value)>>;

    /// Tries to get a [`MapLike`] for a map represented by this `DynamicOps`.
    fn get_map<'a>(
        &'a self,
        input: &'a Self::Value,
    ) -> DataResult<&'a impl MapLike<Value = Self::Value>>;

    /// Gets an [`Iterator`] from a generic value represented by this `DynamicOps`.
    /// This is the equivalent of DFU's `getStream()` function in `DynamicOps`.
    fn get_iter<'a>(
        &self,
        input: &'a Self::Value,
    ) -> DataResult<impl Iterator<Item = &'a Self::Value>>;

    /// Gets a `Vec<i8>` from a generic value represented by this `DynamicOps`.
    /// This is the equivalent of DFU's `getByteBuffer()` function in `DynamicOps`.
    fn get_bytes(&self, input: &Self::Value) -> DataResult<Vec<i8>>
    where
        Self::Value: Display,
    {
        self.get_iter(input).flat_map(|mut iter| {
            // Check if all elements in this value are numbers.
            let all_numbers = iter.all(|e| self.get_number(e).is_success());
            if all_numbers {
                let mut buffer = vec![];
                for e in iter {
                    // This won't panic as we know all values are numbers.
                    let num = self.get_number(e).unwrap();
                    buffer.push(num.into());
                }
                DataResult::success(buffer)
            } else {
                DataResult::error(format!("Some elements are not bytes: {input}"))
            }
        })
    }

    /// Creates a byte list that can be represented by this `DynamicOps` using a byte buffer.
    fn create_byte_list(&self, buffer: Vec<i8>) -> Self::Value {
        self.create_list(buffer.iter().map(|b| self.create_byte(*b)))
    }

    /// Gets an `int` (`i32` in Rust) [`Iterator`] from a generic value represented by this `DynamicOps`.
    /// This is the equivalent of DFU's `getIntStream()` function in `DynamicOps`.
    fn get_int_iter(&self, input: &Self::Value) -> DataResult<impl Iterator<Item = i32>>
    where
        Self::Value: Display,
    {
        self.get_iter(input).flat_map(|mut iter| {
            // Check if all elements in this value are numbers.
            let all_numbers = iter.all(|e| self.get_number(e).is_success());
            if all_numbers {
                DataResult::success(iter.map(|e| {
                    let num = self.get_number(e).unwrap();
                    num.into()
                }))
            } else {
                DataResult::error(format!("Some elements are not ints: {input}"))
            }
        })
    }

    /// Creates an `int` list that can be represented by this `DynamicOps` using a byte buffer.
    fn create_int_list(&self, int_iter: impl Iterator<Item = i32>) -> Self::Value {
        self.create_list(int_iter.map(|i| self.create_int(i)))
    }

    /// Gets a `long` (`i32` in Rust) [`Iterator`] from a generic value represented by this `DynamicOps`.
    /// This is the equivalent of DFU's `getIntStream()` function in `DynamicOps`.
    fn get_long_iter(&self, input: &Self::Value) -> DataResult<impl Iterator<Item = i64>>
    where
        Self::Value: Display,
    {
        self.get_iter(input).flat_map(|mut iter| {
            // Check if all elements in this value are numbers.
            let all_numbers = iter.all(|e| self.get_number(e).is_success());
            if all_numbers {
                DataResult::success(iter.map(|e| {
                    let num = self.get_number(e).unwrap();
                    num.into()
                }))
            } else {
                DataResult::error(format!("Some elements are not longs: {input}"))
            }
        })
    }

    /// Creates a `long` list that can be represented by this `DynamicOps` using a byte buffer.
    fn create_long_list(&self, int_iter: impl Iterator<Item = i64>) -> Self::Value {
        self.create_list(int_iter.map(|l| self.create_long(l)))
    }

    /// Merges a value represented by this `DynamicOps` to a list represented by this `DynamicOps`.
    /// This is only valid if `list` is an actual list.
    fn merge_into_list(&self, list: Self::Value, value: Self::Value) -> DataResult<Self::Value>;

    /// Merges a list of values represented by this `DynamicOps` to another such list.
    /// This is only valid if `list` is an actual list.
    fn merge_values_into_list<I>(&self, list: Self::Value, values: I) -> DataResult<Self::Value>
    where
        I: IntoIterator<Item = Self::Value>,
    {
        let mut result = DataResult::success(list);

        for value in values {
            result = result.flat_map(|list_value| self.merge_into_list(list_value, value));
        }

        result
    }

    /// Merges a value represented by this `DynamicOps` to a list represented by this `DynamicOps`.
    /// This is only valid if `map` is an actual map or is empty. This returns a new map.
    fn merge_into_map(
        &self,
        map: &Self::Value,
        key: Self::Value,
        value: Self::Value,
    ) -> DataResult<Self::Value>
    where
        Self::Value: Clone;

    /// Merges a map represented by this `DynamicOps` to another such map.
    /// This is only valid if `map` is an actual map or is empty. This returns a new map.
    fn merge_entries_into_map<I>(&self, map: &Self::Value, entries: I) -> DataResult<Self::Value>
    where
        I: IntoIterator<Item = (Self::Value, Self::Value)>,
        Self::Value: Clone,
    {
        let mut result = DataResult::success(map.clone());

        for (key, value) in entries {
            result = result.flat_map(|list_value| self.merge_into_map(&list_value, key, value));
        }

        result
    }

    /// Merges a map-like represented by this `DynamicOps` to another such map.
    /// This is only valid if `map` is an actual map or is empty. This returns a new map.
    fn merge_map_like_into_map<M>(
        &self,
        map: &Self::Value,
        other_map_like: &M,
    ) -> DataResult<Self::Value>
    where
        M: MapLike<Value = Self::Value>,
        Self::Value: Clone,
    {
        let mut result = DataResult::success(map.clone());

        for (key, value) in other_map_like.entries() {
            result = result.flat_map(|list_value| {
                self.merge_into_map(&list_value, key.clone(), value.clone())
            });
        }

        result
    }

    /// Merges a value represented by this `DynamicOps` to a primitive type.
    fn merge_into_primitive(
        &self,
        prefix: Self::Value,
        value: Self::Value,
    ) -> DataResult<Self::Value>
    where
        <Self as DynamicOps>::Value: PartialEq + Display,
    {
        if prefix == self.empty() {
            DataResult::success(value)
        } else {
            DataResult::error(format!(
                "Do not know how to append a primitive value {value} to {prefix}"
            ))
        }
    }

    /// Tries to remove something from a value represented by this `DynamicOps` using a key.
    fn remove(&self, input: &mut Self::Value, key: &str);

    /// Whether maps should be compressed under this `DynamicOps`.
    fn compress_maps(&self);

    /// Tries to get a value from a value represented by this `DynamicOps` using a key.
    /// Only works for values that can be [`MapLike`]-viewed.
    fn get_element<'a>(&'a self, input: &'a Self::Value, key: &str) -> DataResult<&'a Self::Value>
    where
        Self::Value: Display,
    {
        self.get_element_generic(input, self.create_string(key.to_string()))
    }

    /// Tries to get a value from a value represented by this `DynamicOps` using a key also represented by this `DynamicOps`.
    /// Only works for values that can be [`MapLike`]-viewed.
    fn get_element_generic<'a>(
        &'a self,
        input: &'a Self::Value,
        key: Self::Value,
    ) -> DataResult<&'a Self::Value>
    where
        Self::Value: Display,
    {
        self.get_map(input).flat_map(|map| {
            map.get(&key).map_or_else(
                || DataResult::error(format!("No element {key} in the map")),
                DataResult::success,
            )
        })
    }

    /// Tries to set a value represented by this `DynamicOps` to a key to a map also represented by this `DynamicOps`.
    /// - It this was successful, this returns the new map value.
    /// - Otherwise, this simply returns `input`.
    fn set_element(&self, input: &Self::Value, key: &str, value: Self::Value) -> Self::Value
    where
        Self::Value: Clone,
    {
        self.merge_into_map(input, self.create_string(key.to_owned()), value)
            .into_result()
            .unwrap_or(input.clone())
    }

    /// Tries to update a value represented by this `DynamicOps` of a map also represented by this `DynamicOps`, with
    /// a key and a mapper function (`f`) whose return value will be the new key's value.
    /// - It this was successful, this returns the newly manipulated map.
    /// - Otherwise, this simply returns `input`.
    fn update_element<F>(&self, input: Self::Value, key: &str, f: F) -> Self::Value
    where
        Self::Value: Display + Clone,
        F: FnOnce(&Self::Value) -> Self::Value,
    {
        self.get_element(&input, key)
            .map(|v| self.set_element(&input, key, f(v)))
            .into_result()
            .unwrap_or(input)
    }

    /// Tries to update a value represented by this `DynamicOps` of a map also represented by this `DynamicOps`, with
    /// a key also represented by this `DynamicOps` and a mapper function (`f`) whose return value will be the new key's value.
    /// - It this was successful, this returns the newly manipulated map.
    /// - Otherwise, this simply returns `input`.
    fn update_element_generic<F>(&self, input: Self::Value, key: Self::Value, f: F) -> Self::Value
    where
        Self::Value: Display + Clone,
        F: FnOnce(&Self::Value) -> Self::Value,
    {
        self.get_element_generic(&input, key.clone())
            .flat_map(|v| self.merge_into_map(&input, key, f(v)))
            .into_result()
            .unwrap_or(input)
    }

    /// Converts a value represented by this `DynamicOps` to another value represented by another `DynamicOps`.
    fn convert_to<U>(&self, out_ops: &impl DynamicOps<Value = U>, input: &Self::Value) -> U;

    /// Converts a list represented by this `DynamicOps` to another list represented by another `DynamicOps`.
    fn convert_list<U>(&self, out_ops: &impl DynamicOps<Value = U>, input: &Self::Value) -> U {
        out_ops.create_list(
            self.get_iter(input)
                .into_result()
                .into_iter()
                .flatten()
                .map(|v| self.convert_to(out_ops, v)),
        )
    }

    /// Converts a map represented by this `DynamicOps` to another map represented by another `DynamicOps`.
    fn convert_map<U>(&self, out_ops: &impl DynamicOps<Value = U>, input: &Self::Value) -> U {
        out_ops.create_map(
            self.get_map_iter(input)
                .into_result()
                .into_iter()
                .flatten()
                .map(|(k, v)| (self.convert_to(out_ops, k), self.convert_to(out_ops, v))),
        )
    }
}
