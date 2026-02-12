use crate::serialization::data_result::DataResult;
use crate::serialization::dynamic_ops::DynamicOps;
use crate::serialization::lifecycle::Lifecycle;

/// A trait specifying a builder to add key-value pairs in order to create a composite type.
///
/// `Value` is the dynamic type for this builder.
/// For a struct, some methods here can be implemented via the `impl_struct_builder` macro.
pub trait StructBuilder {
    type Value;

    /// Adds a single key-value pair to this builder.
    fn add_key_value(&mut self, key: Self::Value, value: Self::Value);

    /// Adds a single key-'value result' pair to this builder.
    fn add_key_value_result(&mut self, key: Self::Value, value: DataResult<Self::Value>);

    /// Adds a single 'key result'-'value result' pair to this builder.
    fn add_key_result_value_result(
        &mut self,
        key: DataResult<Self::Value>,
        value: DataResult<Self::Value>,
    );

    /// Adds the error message from a provided `DataResult` (if any) to this builder internally.
    fn add_errors_from(&mut self, result: DataResult<()>);

    /// Adds a string key-value pair to this builder.
    fn add_string_key_value(&mut self, key: &str, value: Self::Value);

    /// Adds a string key-'value result' pair to this builder.
    fn add_string_key_value_result(&mut self, key: &str, value: DataResult<Self::Value>);

    /// Sets the lifecycle of this builder.
    fn set_lifecycle(&mut self, lifecycle: Lifecycle);

    /// Maps the error from the internal builder to the function `f`.
    fn map_error(&mut self, f: Box<dyn FnOnce(String) -> String>);

    /// Builds the map stored in this builder along with a prefix and returns the result.
    fn build(self, prefix: Self::Value) -> DataResult<Self::Value>;

    /// Builds the map stored in this builder along with a `DataResult` prefix and returns the result.
    fn build_with_result_prefix(self, prefix: DataResult<Self::Value>) -> DataResult<Self::Value>
    where
        Self: Sized,
    {
        prefix.flat_map(|p| self.build(p))
    }
}

/// A [`StructBuilder`] with a specified `Result` type for builders.
pub trait ResultStructBuilder: StructBuilder {
    type Result;

    /// Builds the map stored in `builder` along with a prefix and returns the result.
    fn build_with_builder(
        self,
        builder: Self::Result,
        prefix: Self::Value,
    ) -> DataResult<Self::Value>;
}

/// A subtrait of [`StructBuilder`] for appending string keys instead of dynamic type keys.
/// The methods in `StructBuilder` can also be implemented via the `impl_string_struct_builder` macro.
pub trait StringStructBuilder: ResultStructBuilder {
    /// Appends a string key-value pair to `builder`, mutating it.
    fn append(&self, key: &str, value: Self::Value, builder: Self::Result) -> Self::Result;
}

/// A subtrait of [`StructBuilder`] for appending dynamic keys. The methods in `StructBuilder`
/// can also be implemented via the `impl_universal_struct_builder` macro.
pub trait UniversalStructBuilder: ResultStructBuilder {
    /// Appends a key-value pair to `builder`, mutating it.
    fn append(&self, key: Self::Value, value: Self::Value, builder: Self::Result) -> Self::Result;
}

/// A macro to be placed inside an `impl` block of a struct implementing [`StructBuilder`].
///
/// Place this in a `impl StructBuilder for ...` block.
/// This automatically implements the methods to add key-value pairs to the builder.
/// Make sure to have a struct field of type [`DataResult<Self::Value>`] of name `$builder`.
#[macro_export]
macro_rules! impl_struct_builder {
    ($builder:ident) => {
        fn set_lifecycle(&mut self, lifecycle: Lifecycle) {
            self.$builder = self.$builder.clone().with_lifecycle(lifecycle);
        }

        fn map_error(&mut self, f: Box<dyn FnOnce(String) -> String>) {
            self.$builder = self.$builder.clone().map_error_dyn(f);
        }

        fn add_errors_from(&mut self, result: DataResult<()>) {
            self.$builder = self.$builder.clone().flat_map(|v| result.map(|_| v));
        }

        fn build(self, prefix: Self::Value) -> DataResult<Self::Value> {
            self.$builder
                .clone()
                .flat_map(|b| self.build_with_builder(b, prefix))
        }
    };
}

/// A macro to be placed inside an `impl` block of a struct implementing [`StringStructBuilder`].
///
/// Place this in a `impl StructBuilder for ...` block.
/// This automatically implements the methods to add key-value pairs to the builder.
#[macro_export]
macro_rules! impl_string_struct_builder {
    (@internal $builder:ident) => {
        fn add_string_key_value(&mut self, key: &str, value: Self::Value) {
            self.$builder = self.$builder.clone().map(|r| self.append(key, value, r))
        }

        fn add_string_key_value_result(&mut self, key: &str, value: DataResult<Self::Value>) {
            self.$builder = self.$builder.clone().apply_2_and_make_stable(|r, v| self.append(key, v, r), value);
        }
    };

    // For constant ops
    ($builder:ident, $ops:ident) => {

        impl_string_struct_builder!(@internal $builder);

        fn add_key_value(&mut self, key: Self::Value, value: Self::Value) {
            self.$builder = $ops.get_string(&key).flat_map(|s| {
                self.add_string_key_value(&*s, value);
                return self.$builder.clone();
            })
        }

        fn add_key_value_result(&mut self, key: Self::Value, value: DataResult<Self::Value>) {
            self.$builder = $ops.get_string(&key).flat_map(|s| {
                self.add_string_key_value_result(&*s, value);
                return self.$builder.clone();
            })
        }

        fn add_key_result_value_result(
            &mut self,
            key: DataResult<Self::Value>,
            value: DataResult<Self::Value>,
        ) {
            self.$builder = key.flat_map(|v| $ops.get_string(&v)).flat_map(|s| {
                self.add_string_key_value_result(&*s, value);
                return self.$builder.clone();
            })
        }
    };

    // For stored ops
    ($builder:ident, self. $ops:ident) => {

        impl_string_struct_builder!(@internal $builder);

        fn add_key_value(&mut self, key: Self::Value, value: Self::Value) {
            self.$builder = self.$ops.get_string(&key).flat_map(|s| {
                self.add_string_key_value(&*s, value);
                return self.$builder.clone();
            })
        }

        fn add_key_value_result(&mut self, key: Self::Value, value: DataResult<Self::Value>) {
            self.$builder = self.$ops.get_string(&key).flat_map(|s| {
                self.add_string_key_value_result(&*s, value);
                return self.$builder.clone();
            })
        }

        fn add_key_result_value_result(
            &mut self,
            key: DataResult<Self::Value>,
            value: DataResult<Self::Value>,
        ) {
            self.$builder = key.flat_map(|v| self.$ops.get_string(&v)).flat_map(|s| {
                self.add_string_key_value_result(&*s, value);
                return self.$builder.clone();
            })
        }
    };
}

/// A macro to be placed inside an `impl` block of a struct implementing `UniversalStructBuilder`.
///
/// Place this in a `impl StructBuilder for ...` block.
/// This automatically implements the methods to add key-value pairs to the builder.
#[macro_export]
macro_rules! impl_universal_struct_builder {
    (@internal $builder:ident) => {
        fn add_key_value(&mut self, key: Self::Value, value: Self::Value) {
            self.$builder = self.$builder.clone().map(|b| self.append(key, value, b))
        }

        fn add_key_value_result(&mut self, key: Self::Value, value: DataResult<Self::Value>) {
            self.$builder = self
                .$builder
                .clone()
                .apply_2_and_make_stable(|b, v| self.append(key, v, b), value);
        }

        fn add_key_result_value_result(
            &mut self,
            key: DataResult<Self::Value>,
            value: DataResult<Self::Value>,
        ) {
            self.$builder = self
                .$builder
                .clone()
                .apply(key.apply_2_and_make_stable(|k, v| (|b| self.append(k, v, b)), value));
        }
    };

    // For constant ops
    ($builder:ident, $ops:ident) => {
        impl_universal_struct_builder!(@internal $builder);

        fn add_string_key_value(&mut self, key: &str, value: Self::Value) {
            self.add_key_value($ops.create_string(key), value);
        }

        fn add_string_key_value_result(&mut self, key: &str, value: DataResult<Self::Value>) {
            self.add_key_value_result($ops.create_string(key), value);
        }
    };

    // For stored ops
    ($builder:ident, self. $ops:ident) => {
        impl_universal_struct_builder!(@internal $builder);

        fn add_string_key_value(&mut self, key: &str, value: Self::Value) {
            self.add_key_value(self.$ops.create_string(key), value);
        }

        fn add_string_key_value_result(&mut self, key: &str, value: DataResult<Self::Value>) {
            self.add_key_value_result(self.$ops.create_string(key), value);
        }
    };
}

pub struct MapBuilder<T, O: DynamicOps<Value = T> + 'static> {
    builder: DataResult<Vec<(T, T)>>,
    ops: &'static O,
}

impl<T: Clone, O: DynamicOps<Value = T>> MapBuilder<T, O> {
    pub(crate) const fn new(ops: &'static O) -> Self {
        Self {
            builder: DataResult::success_with_lifecycle(vec![], Lifecycle::Stable),
            ops,
        }
    }
}

impl<T: Clone, O: DynamicOps<Value = T>> StructBuilder for MapBuilder<T, O> {
    type Value = T;

    impl_struct_builder!(builder);
    impl_universal_struct_builder!(builder, self.ops);
}

impl<T: Clone, O: DynamicOps<Value = T>> ResultStructBuilder for MapBuilder<T, O> {
    type Result = Vec<(T, T)>;

    fn build_with_builder(
        self,
        builder: Self::Result,
        prefix: Self::Value,
    ) -> DataResult<Self::Value> {
        self.ops.merge_entries_into_map(prefix, builder)
    }
}

impl<T: Clone, O: DynamicOps<Value = T>> UniversalStructBuilder for MapBuilder<T, O> {
    fn append(
        &self,
        key: Self::Value,
        value: Self::Value,
        mut builder: Self::Result,
    ) -> Self::Result {
        builder.push((key, value));
        builder
    }
}
