use crate::serialization::{Number, data_result::DataResult, map_like::MapLike};

/// Generates default implementations for numeric creators.
macro_rules! numeric_creators {
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

    /// Converts a value represented by this `DynamicOps` to another value represented by another `DynamicOps`.
    fn convert_to<U>(out_ops: impl DynamicOps<Value = U>, input: Self::Value);

    /// Returns how an empty value is represented by this `DynamicOps`.
    fn empty() -> Self::Value;

    /// Returns how a generic number is represented by this `DynamicOps`.
    fn create_number(&self, n: Number) -> Self::Value;

    numeric_creators! {
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

    /// Tries to get a number represented by this `DynamicOps`.
    fn get_number(&self, input: Self::Value) -> DataResult<Number>;

    /// Tries to get a string represented by this `DynamicOps`.
    fn get_string(&self, input: Self::Value) -> DataResult<String>;

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
    /// This is only valid if `map` is an actual map.
    fn merge_into_map(
        &self,
        map: Self::Value,
        key: Self::Value,
        value: Self::Value,
    ) -> DataResult<Self::Value>;

    /// Merges a map represented by this `DynamicOps` to another such map.
    /// This is only valid if `map` is an actual map.
    fn merge_entries_into_map<I>(&self, map: Self::Value, entries: I) -> DataResult<Self::Value>
    where
        I: IntoIterator<Item = (Self::Value, Self::Value)>,
    {
        let mut result = DataResult::success(map);

        for (key, value) in entries {
            result = result.flat_map(|list_value| self.merge_into_map(list_value, key, value));
        }

        result
    }

    /// Merges a map-like represented by this `DynamicOps` to another such map.
    /// This is only valid if `map` is an actual map.
    fn merge_map_like_into_map<'a, M>(
        &self,
        map: Self::Value,
        other_map_like: &M,
    ) -> DataResult<Self::Value>
    where
        M: MapLike<'a, Value = Self::Value>,
        Self::Value: Clone,
        <Self as DynamicOps>::Value: 'a,
    {
        let mut result = DataResult::success(map);

        for (key, value) in other_map_like.entries() {
            result = result
                .flat_map(|list_value| self.merge_into_map(list_value, key.clone(), value.clone()));
        }

        result
    }
}
