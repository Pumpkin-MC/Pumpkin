use crate::serialization::{
    HasValue,
    codec::Codec,
    coders::{Decoder, Encoder},
    data_result::DataResult,
    dynamic_ops::DynamicOps,
};

/// Helper macro to generate the struct and [`HasValue`] trait implementation for a `PrimitiveCodec` struct.
macro_rules! impl_primitive_codec_start {
    ($name:ident, $prim:ty) => {
        pub struct $name;

        impl HasValue for $name {
            type Value = $prim;
        }
    };
}

/// Helper macro to generate an entire implementation for a number `PrimitiveCodec`.
macro_rules! impl_primitive_number_codec {
    ($name:ident, $prim:ty, $create_func:ident) => {
        impl_primitive_codec_start!($name, $prim);
        impl PrimitiveCodec for $name {
            fn read<T>(
                &self,
                ops: &'static impl DynamicOps<Value = T>,
                input: T,
            ) -> DataResult<$prim> {
                ops.get_number(&input).map(|n| <$prim>::from(n))
            }

            fn write<T>(&self, ops: &'static impl DynamicOps<Value = T>, value: &$prim) -> T {
                ops.$create_func(*value)
            }
        }
    };
}

/// Helper macro to generate an entire implementation for a list `PrimitiveCodec`.
macro_rules! impl_primitive_list_codec {
    ($name:ident, $elem:ty, $get_func:ident, $create_func:ident) => {
        impl_primitive_codec_start!($name, Vec<$elem>);
        impl PrimitiveCodec for $name {
            fn read<T>(
                &self,
                ops: &'static impl DynamicOps<Value = T>,
                input: T,
            ) -> DataResult<Vec<$elem>> {
                ops.$get_func(&input)
            }

            fn write<T>(&self, ops: &'static impl DynamicOps<Value = T>, value: &Vec<$elem>) -> T {
                ops.$create_func(value.to_vec())
            }
        }
    };
}

/// A generic primitive codec.
trait PrimitiveCodec: Codec {
    fn read<T>(
        &self,
        ops: &'static impl DynamicOps<Value = T>,
        input: T,
    ) -> DataResult<Self::Value>;

    fn write<T>(&self, ops: &'static impl DynamicOps<Value = T>, value: &Self::Value) -> T;
}

impl<C: PrimitiveCodec> Encoder for C {
    fn encode<T: PartialEq>(
        &self,
        input: &<C as HasValue>::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T> {
        ops.merge_into_primitive(prefix, self.write(ops, input))
    }
}

impl<C: PrimitiveCodec> Decoder for C {
    fn decode<T: PartialEq>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(<C as HasValue>::Value, T)> {
        self.read(ops, input).map(|r| (r, ops.empty()))
    }
}

// Implementations

impl_primitive_codec_start!(BoolCodec, bool);
impl PrimitiveCodec for BoolCodec {
    fn read<T>(&self, ops: &'static impl DynamicOps<Value = T>, input: T) -> DataResult<bool> {
        ops.get_bool(&input)
    }

    fn write<T>(&self, ops: &'static impl DynamicOps<Value = T>, value: &bool) -> T {
        ops.create_bool(*value)
    }
}

impl_primitive_number_codec!(ByteCodec, i8, create_byte);
impl_primitive_number_codec!(ShortCodec, i16, create_short);
impl_primitive_number_codec!(IntCodec, i32, create_int);
impl_primitive_number_codec!(LongCodec, i64, create_long);
impl_primitive_number_codec!(FloatCodec, f32, create_float);
impl_primitive_number_codec!(DoubleCodec, f64, create_double);

impl_primitive_codec_start!(StringCodec, String);
impl PrimitiveCodec for StringCodec {
    fn read<T>(&self, ops: &'static impl DynamicOps<Value = T>, input: T) -> DataResult<String> {
        ops.get_string(&input)
    }

    fn write<T>(&self, ops: &'static impl DynamicOps<Value = T>, value: &String) -> T {
        ops.create_string(value)
    }
}

impl_primitive_list_codec!(ByteBufferCodec, i8, get_byte_buffer, create_byte_buffer);
impl_primitive_list_codec!(IntStreamCodec, i32, get_int_list, create_int_list);
impl_primitive_list_codec!(LongStreamCodec, i64, get_long_list, create_long_list);

/// A primitive codec for Java's `boolean` (`bool` in Rust).
pub const BOOL: BoolCodec = BoolCodec;

/// A primitive codec for Java's `byte` (or `i8` in Rust).
pub const BYTE: ByteCodec = ByteCodec;
/// A primitive codec for Java's `short` (or `i16` in Rust).
pub const SHORT: ShortCodec = ShortCodec;
/// A primitive codec for Java's `int` (or `i32` in Rust).
pub const INT: IntCodec = IntCodec;
/// A primitive codec for Java's `long` (or `i64` in Rust).
pub const LONG: LongCodec = LongCodec;
/// A primitive codec for Java's `float` (or `f32` in Rust).
pub const FLOAT: FloatCodec = FloatCodec;
/// A primitive codec for Java's `double` (or `f64` in Rust).
pub const DOUBLE: DoubleCodec = DoubleCodec;

/// A primitive codec for Java's `String` (also `String` in Rust).
pub const STRING: StringCodec = StringCodec;

/// A primitive codec for Java's `ByteBuffer`.
/// Here, this actually stores a [`Vec<i8>`].
pub const BYTE_BUFFER: ByteBufferCodec = ByteBufferCodec;
/// A primitive codec for Java's `IntStream`.
/// Here, this actually stores a [`Vec<i32>`].
pub const INT_STREAM: IntStreamCodec = IntStreamCodec;
/// A primitive codec for Java's `LongStream`.
/// Here, this actually stores a [`Vec<i64>`].
pub const LONG_STREAM: LongStreamCodec = LongStreamCodec;
