use std::{collections::HashMap, fmt::Display};

use crate::serialization::struct_builder::ResultStructBuilder;
use crate::serialization::{
    Number,
    data_result::DataResult,
    list_builder::{ListBuilder, new_list_builder_impl},
    map_like::MapLike,
};

/// Generates a default implementation for a create_*number* function.
macro_rules! create_number_impl {
    ($name:ident, $ty:ty, $number_ty:ident, $func:ident) => {
        /// Returns how a generic
        #[doc = concat!("`", stringify!($name), "`")]
        /// in Java (equivalent to
        #[doc = concat!("[`", stringify!($ty), "`])")]
        /// is represented by this `DynamicOps`.
        fn $func(&self, data: $ty) -> Self::Value {
            self.create_number(Number::$number_ty(data))
        }
    };
}

/// A macro providing a default implementation of a get_... function in `DynamicOps`.
///
/// Those functions include:
/// - [`DynamicOps::get_byte_buffer`] (put `box` before the target expression (usually self))
/// - [`DynamicOps::get_int_list`]
/// - [`DynamicOps::get_long_list`]
#[macro_export]
macro_rules! impl_get_list {
    (box $target:expr, $input:expr, $ty:literal) => {
        $target.get_iter($input).flat_map(|iter| {
            // We want all elements in the iterator to be numbers.
            iter.map(|e| $target.get_number(&e).into_result().map(Into::into))
                .collect::<Option<Vec<_>>>()
                .map_or_else(
                    || DataResult::error(concat!("Some elements are not ", $ty).to_string()),
                    |v| DataResult::success(v.into_boxed_slice()),
                )
        })
    };
    ($target:expr, $input:expr, $ty:literal) => {
        $target.get_iter($input).flat_map(|iter| {
            // We want all elements in the iterator to be numbers.
            iter.map(|e| $target.get_number(&e).into_result().map(Into::into))
                .collect::<Option<Vec<_>>>()
                .map_or_else(
                    || DataResult::error(concat!("Some elements are not ", $ty).to_string()),
                    DataResult::success,
                )
        })
    };
}

/// A trait describing methods to read and write a specific format (like NBT or JSON).
/// The `Value` of this trait is the type that can be used to represent anything in this format.
pub trait DynamicOps {
    type Value: PartialEq + Display + Clone;
    type StructBuilder: ResultStructBuilder<Value = Self::Value>;

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

    create_number_impl!(byte, i8, Byte, create_byte);
    create_number_impl!(short, i16, Short, create_short);
    create_number_impl!(int, i32, Int, create_int);
    create_number_impl!(long, i64, Long, create_long);
    create_number_impl!(float, f32, Float, create_float);
    create_number_impl!(double, f64, Double, create_double);

    /// Returns how a boolean is represented by this `DynamicOps`.
    fn create_bool(&self, data: bool) -> Self::Value {
        self.create_byte(i8::from(data))
    }

    /// Returns how a string is represented by this `DynamicOps`.
    fn create_string(&self, data: &str) -> Self::Value;

    /// Returns how a list is represented by this `DynamicOps`.
    fn create_list<I>(&self, values: I) -> Self::Value
    where
        I: IntoIterator<Item = Self::Value>;

    /// Returns how a map is represented by this `DynamicOps`.
    fn create_map<I>(&self, entries: I) -> Self::Value
    where
        I: IntoIterator<Item = (Self::Value, Self::Value)>;

    /// Tries to get a `bool` represented by this `DynamicOps`.
    fn get_bool(&self, input: &Self::Value) -> DataResult<bool> {
        self.get_number(input).map(|n| i8::from(n) != 0)
    }

    /// Tries to get a number represented by this `DynamicOps`.
    fn get_number(&self, input: &Self::Value) -> DataResult<Number>;

    /// Tries to get a string represented by this `DynamicOps`.
    fn get_string(&self, input: &Self::Value) -> DataResult<String>;

    /// Gets an [`Iterator`] of key-value pairs from a map represented by this `DynamicOps`.
    /// This is only applicable to map-like values.
    fn get_map_iter<'a>(
        &'a self,
        input: &'a Self::Value,
    ) -> DataResult<impl Iterator<Item = (Self::Value, &'a Self::Value)> + 'a>;

    /// Tries to get a [`MapLike`] for a map represented by this `DynamicOps`.
    fn get_map<'a>(
        &self,
        input: &'a Self::Value,
    ) -> DataResult<impl MapLike<Value = Self::Value> + 'a>;

    /// Gets an [`Iterator`] from a generic value represented by this `DynamicOps`.
    /// This is the equivalent of DFU's `getStream()` function, and is only applicable to list-like values.
    fn get_iter(&self, input: Self::Value) -> DataResult<impl Iterator<Item = Self::Value>>;

    /// Gets a `Box<[u8]>` (byte buffer) from a generic value represented by this `DynamicOps`.
    /// This is the equivalent of DFU's `getByteBuffer()` function.
    fn get_byte_buffer(&self, input: Self::Value) -> DataResult<Box<[u8]>> {
        impl_get_list!(box self, input, "bytes")
    }

    /// Creates a byte buffer that can be represented by this `DynamicOps` using a [`Vec<u8>`].
    fn create_byte_buffer(&self, buffer: Vec<u8>) -> Self::Value {
        self.create_list(buffer.iter().map(|b| self.create_byte(*b as i8)))
    }

    /// Gets a [`Vec<i32>`] (`int` list) from a generic value represented by this `DynamicOps`.
    /// This is the equivalent of DFU's `getIntStream()` function.
    fn get_int_list(&self, input: Self::Value) -> DataResult<Vec<i32>> {
        impl_get_list!(self, input, "ints")
    }

    /// Creates an `int` list ([`Vec<i32>`]) that can be represented by this `DynamicOps`.
    fn create_int_list(&self, vec: Vec<i32>) -> Self::Value {
        self.create_list(vec.into_iter().map(|i| self.create_int(i)))
    }

    /// Gets a [`Vec<i64>`] (`long` list) from a generic value represented by this `DynamicOps`.
    /// This is the equivalent of DFU's `getLongStream()` function.
    fn get_long_list(&self, input: Self::Value) -> DataResult<Vec<i64>> {
        impl_get_list!(self, input, "longs")
    }

    /// Creates a `long` list ([`Vec<i64>`]) that can be represented by this `DynamicOps`.
    fn create_long_list(&self, vec: Vec<i64>) -> Self::Value {
        self.create_list(vec.into_iter().map(|l| self.create_long(l)))
    }

    /// Merges a value represented by this `DynamicOps` to a list represented by this `DynamicOps`.
    /// This is only valid if `list` is an actual list.
    fn merge_into_list(&self, list: Self::Value, value: Self::Value) -> DataResult<Self::Value>;

    /// Merges a list of values represented by this `DynamicOps` into another such list.
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

    /// Adds a key-value pair, both represented by this `DynamicOps`, to a map also represented by this `DynamicOps`,
    /// returning the new map. This is only valid if `map` is an actual map or is empty.
    fn merge_into_map(
        &self,
        map: Self::Value,
        key: Self::Value,
        value: Self::Value,
    ) -> DataResult<Self::Value>
    where
        Self::Value: Clone;

    /// Merges a map represented by this `DynamicOps` into another such map, returning the new map.
    /// This is only valid if `map` is an actual map or is empty.
    fn merge_entries_into_map<I>(&self, map: Self::Value, entries: I) -> DataResult<Self::Value>
    where
        I: IntoIterator<Item = (Self::Value, Self::Value)>,
        Self::Value: Clone,
    {
        let mut result = DataResult::success(map);

        for (key, value) in entries {
            result = result.flat_map(|list_value| self.merge_into_map(list_value, key, value));
        }

        result
    }

    /// Merges a [`MapLike`] represented by this `DynamicOps` into another such map, returning the new map.
    /// This is only valid if `map` is an actual map or is empty.
    fn merge_map_like_into_map<M>(
        &self,
        map: Self::Value,
        other_map_like: M,
    ) -> DataResult<Self::Value>
    where
        M: MapLike<Value = Self::Value>,
        Self::Value: Clone,
    {
        let mut result = DataResult::success(map);

        for (key, value) in other_map_like.iter() {
            result =
                result.flat_map(|list_value| self.merge_into_map(list_value, key, value.clone()));
        }

        result
    }

    /// Merges a value represented by this `DynamicOps` into a primitive type.
    fn merge_into_primitive(
        &self,
        prefix: Self::Value,
        value: Self::Value,
    ) -> DataResult<Self::Value>
    where
        <Self as DynamicOps>::Value: PartialEq,
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
    /// This returns the new value if successful, otherwise, this returns itself.
    fn remove(&self, input: Self::Value, key: &str) -> Self::Value;

    /// Whether maps should be compressed under this `DynamicOps`.
    fn compress_maps(&self) -> bool {
        false
    }

    /// Tries to get a value from a value represented by this `DynamicOps` using a key.
    /// Only works for values that can be [`MapLike`]-viewed.
    fn get_element<'a>(&'a self, input: &'a Self::Value, key: &str) -> DataResult<&'a Self::Value> {
        self.get_element_generic(input, &self.create_string(key))
    }

    /// Tries to get a value from a value represented by this `DynamicOps` using a key also represented by this `DynamicOps`.
    fn get_element_generic<'a>(
        &'a self,
        input: &'a Self::Value,
        key: &Self::Value,
    ) -> DataResult<&'a Self::Value>
where {
        self.get_map_iter(input).flat_map(|mut iter| {
            iter.find(|(k, _)| k == key).map_or_else(
                || DataResult::error(format!("No element {key} in the map")),
                |(_, v)| DataResult::success(v),
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
        self.merge_into_map(input.clone(), self.create_string(key), value)
            .into_result()
            .unwrap_or(input.clone())
    }

    /// Tries to update a value represented by this `DynamicOps` of a map also represented by this `DynamicOps`, with
    /// a key and a mapper function (`f`) whose return value will be the new value of the specified key.
    /// - It this was successful, this returns the newly manipulated map.
    /// - Otherwise, this simply returns `input`.
    fn update_element<F>(&self, input: &Self::Value, key: &str, f: F) -> Self::Value
    where
        F: FnOnce(&Self::Value) -> Self::Value,
    {
        self.get_element(input, key)
            .map(|v| self.set_element(input, key, f(v)))
            .into_result()
            .unwrap_or(input.clone())
    }

    /// Tries to update a value represented by this `DynamicOps` of a map also represented by this `DynamicOps`, with
    /// a key also represented by this `DynamicOps` and a mapper function (`f`) whose return value will be the new key's value.
    /// - It this was successful, this returns the newly manipulated map.
    /// - Otherwise, this simply returns `input`.
    fn update_element_generic<F>(&self, input: &Self::Value, key: &Self::Value, f: F) -> Self::Value
    where
        F: FnOnce(&Self::Value) -> Self::Value,
    {
        self.get_element_generic(input, key)
            .flat_map(|v| self.merge_into_map(input.clone(), key.clone(), f(v)))
            .into_result()
            .unwrap_or(input.clone())
    }

    /// Converts a value represented by this `DynamicOps` to another value represented by another `DynamicOps`.
    fn convert_to<U>(&self, out_ops: &impl DynamicOps<Value = U>, input: Self::Value) -> U;

    /// Converts a list represented by this `DynamicOps` to another list represented by another `DynamicOps`.
    fn convert_list<U>(&self, out_ops: &impl DynamicOps<Value = U>, input: Self::Value) -> U {
        out_ops.create_list(
            self.get_iter(input)
                .into_result()
                .into_iter()
                .flatten()
                .map(|v| self.convert_to(out_ops, v)),
        )
    }

    /// Converts a map represented by this `DynamicOps` to another map represented by another `DynamicOps`.
    fn convert_map<U>(&self, out_ops: &impl DynamicOps<Value = U>, input: Self::Value) -> U {
        out_ops.create_map(
            self.get_map_iter(&input)
                .into_result()
                .into_iter()
                .flatten()
                .map(|(k, v)| {
                    (
                        self.convert_to(out_ops, k),
                        self.convert_to(out_ops, v.clone()),
                    )
                }),
        )
    }

    /// Returns a [`ListBuilder`] for this `DynamicOps`.
    fn list_builder(&'static self) -> impl ListBuilder<Value = Self::Value>
    where
        Self: Sized,
    {
        new_list_builder_impl(self)
    }

    /// Returns a [`MapBuilder`] for this `DynamicOps`.
    fn map_builder(&'static self) -> Self::StructBuilder;
}
