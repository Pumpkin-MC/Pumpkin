pub use super::map_codec::for_getter;
use crate::serialization::HasValue;
use crate::serialization::codecs::lazy::{LazyCodec, new_lazy_codec};
use crate::serialization::codecs::list::{ListCodec, new_list_codec};
use crate::serialization::codecs::primitive::{
    BoolCodec, ByteBufferCodec, ByteCodec, DoubleCodec, FloatCodec, IntCodec, IntStreamCodec,
    LongCodec, LongStreamCodec, ShortCodec, StringCodec,
};
use crate::serialization::codecs::range::RangeCodec;
use crate::serialization::codecs::range::new_range_codec;
use crate::serialization::codecs::unbounded_map::{UnboundedMapCodec, new_unbounded_map_codec};
use crate::serialization::codecs::validated::{ValidatedCodec, new_validated_codec};
use crate::serialization::coders::{
    ComappedEncoderImpl, Decoder, Encoder, FlatComappedEncoderImpl, FlatMappedDecoderImpl,
    MappedDecoderImpl, comap, decoder_field, encoder_field, flat_comap, flat_map, map,
};
use crate::serialization::data_result::DataResult;
use crate::serialization::dynamic_ops::DynamicOps;
use crate::serialization::keyable::Keyable;
use crate::serialization::map_codec::ComposedMapCodec;
#[allow(unused_imports)] // Only used in documentation.
use crate::serialization::map_codec::MapCodec;
use crate::serialization::map_codecs::field_coders::{FieldDecoder, FieldEncoder};
use crate::serialization::map_codecs::optional_field::{
    DefaultValueProviderMapCodec, OptionalFieldMapCodec, new_default_value_provider_map_codec,
    new_optional_field_map_codec,
};
use crate::serialization::map_codecs::simple::{SimpleMapCodec, new_simple_map_codec};
#[allow(unused_imports)] // Only used in documentation.
use crate::serialization::struct_codecs::Field;
use std::fmt::Display;
use std::hash::Hash;

/// A type of *codec* describing the way to **encode from and decode into** something of a type `Value`  (`Value` -> `?` and `?` -> `Value`).
///
/// # Usage
/// This trait is the main way serialization/deserialization can be handled easily.
/// - To encode something, use [`Codec::encode_start`]
/// - To decode something, use [`Codec::parse`].
///
/// To use these methods, use a [`DynamicOps`] instance to tell the intermediate format to encode to/decode from:
///
/// # Primitive Codecs
/// This trait's module (`codec`) provides many common codecs that can be used for more complex codec types:
/// - [`BYTE_CODEC`], [`SHORT_CODEC`], [`INT_CODEC`], [`LONG_CODEC`], [`BOOL_CODEC`], [`FLOAT_CODEC`] and [`DOUBLE_CODEC`] for Java primitive types.
/// - [`STRING_CODEC`] for `String`s.
/// - [`BYTE_CODEC`], [`USHORT_CODEC`], [`UINT_CODEC`] and [`ULONG_CODEC`] for unsigned versions of Java primitive number types (`u8`, `u16`, `u32` and `u64`).
/// - [`BYTE_BUFFER_CODEC`] for byte buffers (equivalent to `Box<[u8]>`).
/// - [`INT_STREAM_CODEC`] and [`LONG_STREAM_CODEC`] for Java's `int` and `long` stream codecs (equivalent to `Vec<i32>` and `Vec<i64>`).
///
/// # Creating a Codec
/// There are a few codec types that can be created for custom types. **Keep in mind that codecs are meant
/// to be static instances, and they should not be created at runtime.** Usually, codecs are declared
/// using `pub static`.
///
/// ## Lists
/// Use one of the following with the required arguments:
/// - [`list`]: Creates a list codec of a given codec with the provided minimum and maximum size limits.
/// - [`limited_list`]: Creates a list codec of a given codec with the provided maximum size limit.
/// - [`unbounded_list`]: Creates a list codec of a given codec with no size limit.
///
/// ## Ranges
/// A codec can also only accept a range of values of some number type. You can use one of the following for that:
/// - [`int_range`]: For `int`s.
/// - [`float_range`]: For `float`s.
/// - [`double_range`]: For `double`s.
///
/// ## Structs
/// Use the [`crate::struct_codec!`] macro to generate a codec implementation for a struct.
/// A struct codec can work with up to 16 [`Field`]s, which each take a [`MapCodec`]
/// and a getter. A `MapCodec` is simply an object that works with one or more keys of a provided map.
/// Most of them used will be [`FieldMapCodec`]s, which only work with one singular key.
///
/// A field `FieldMapCodec` can be created with one of the following:
/// - [`field`]: Provides a *required* field with the provided codec and name.
/// - [`optional_field`]: Provides an *optional* field with the provided codec and name. Since this type of `MapCodec`
///   has **no default value**, it encodes into an [`Option`].
/// - [`optional_field_with_default`]: Provides an *optional* field with the provided codec and name, along with a default value factory
///   for when the value does not exist while decoding.
/// - [`lenient_optional_field`] and [`lenient_optional_field_with_default`] for lenient versions of the above two optional field methods.
///
/// To create a `Field` object using a `MapCodec`, use [`for_getter`] (in `map_codec`) to include a getter method
/// to tell the codec how to get some value (for encoding) from a struct instance. These `Field`s can then be placed
/// in the `struct_codec` body, one for each pair, along with a constructor function at the end
/// to tell the codec how to create an instance (for decoding) with the provided values. See the documentation
/// of the `struct_codec` macro for a basic example for defining a struct codec.
///
/// ## Unbounded Maps
/// Use the [`unbounded_map`] function to create a codec encoding/decoding a `HashMap` of any arbitrary key.
/// **Unbounded map codecs only support keys that can encode from/decode to strings.**
///
/// # Transformers
/// A map codec of a type `B` can be implemented by *transforming* another codec of type `A` to work with type `B`.
/// The following methods can be used depending on the equivalence relation between the two types:
/// - [`xmap`]
/// - [`comap_flat_map`]
/// - [`flat_map_comap`]
/// - [`flat_xmap`]
///
/// For example, the unsigned types use `flat_xmap` to convert between the `i_` and `u_` types.
///
/// # Validator Codecs
/// The [`validate`] function returns a codec wrapper that validates a value before encoding and after decoding.
/// A validated codec takes a function that can either return an [`Ok`] for a success,
/// or an [`Err`] with the provided message to place in a `DataResult`.
pub trait Codec: Encoder + Decoder {}

// Any struct implementing Encoder<Value = A> and Decoder<Value = A> will also implement Codec<Value = A>.
impl<T> Codec for T where T: Encoder + Decoder {}

/// A codec allowing an arbitrary encoder and decoder.
pub struct ComposedCodec<E: Encoder + 'static, D: Decoder<Value = E::Value> + 'static> {
    encoder: E,
    decoder: D,
}

impl<E: Encoder, D: Decoder<Value = E::Value>> HasValue for ComposedCodec<E, D> {
    type Value = E::Value;
}

impl<E: Encoder, D: Decoder<Value = E::Value>> Encoder for ComposedCodec<E, D> {
    fn encode<T: Display + PartialEq + Clone>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: T,
    ) -> DataResult<T> {
        self.encoder.encode(input, ops, prefix)
    }
}

impl<E: Encoder, D: Decoder<Value = E::Value>> Decoder for ComposedCodec<E, D> {
    fn decode<T: Display + PartialEq + Clone>(
        &self,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<(Self::Value, T)> {
        self.decoder.decode(input, ops)
    }
}

// Primitive codecs

macro_rules! define_const_codec {
    ($name:ident, $codec_ty:ident, $ty:ident, $java_ty:ident) => {
        #[doc = concat!("A primitive codec for Java's `", stringify!($java_ty), "` (`", stringify!($ty), "` in Rust).")]
        pub const $name: $codec_ty = $codec_ty;
    };
    (vec $name:ident, $codec_ty:ident, $vec_ty:ident, $java_ty:ident) => {
        #[doc = concat!("A primitive codec for Java's `", stringify!($java_ty), "`.")]
        #[doc = concat!("Here, this actually stores a [`Vec<", stringify!($vec_ty), ">`].")]
        pub const $name: $codec_ty = $codec_ty;
    };
}

define_const_codec!(BOOL_CODEC, BoolCodec, bool, boolean);

define_const_codec!(BYTE_CODEC, ByteCodec, i8, byte);
define_const_codec!(SHORT_CODEC, ShortCodec, i16, short);
define_const_codec!(INT_CODEC, IntCodec, i32, int);
define_const_codec!(LONG_CODEC, LongCodec, i64, long);
define_const_codec!(FLOAT_CODEC, FloatCodec, f32, float);
define_const_codec!(DOUBLE_CODEC, DoubleCodec, f64, double);

define_const_codec!(STRING_CODEC, StringCodec, String, String);

define_const_codec!(vec BYTE_BUFFER_CODEC, ByteBufferCodec, i8, ByteBuffer);
define_const_codec!(vec INT_STREAM_CODEC, IntStreamCodec, i16, IntStream);
define_const_codec!(vec LONG_STREAM_CODEC, LongStreamCodec, i32, LongStream);

// Unsigned types

/// Helper macro to generate a [`Codec`] of unsigned number types using `flat_xmap` of their signed counterparts.
macro_rules! impl_unsigned_transformer_codec {
    ($name:ident, $signed_codec_type:ident, $unsigned_codec_type:ident, $unsigned_prim:ident, $signed_prim:ident, $transformed_codec:ident) => {
        #[doc = concat!("The codec type for the [`", stringify!($unsigned_prim), "`] data type.")]
        pub type $unsigned_codec_type = FlatXmapCodec<$unsigned_prim, $signed_codec_type>;

        #[doc = concat!("A [`Codec`] for `", stringify!($unsigned_prim), "`, which is a transformer codec of [`", stringify!($transformed_codec), "`].")]
        ///
        /// Be wary that
        #[doc = concat!("if any encoded value exceeds [`", stringify!($signed_prim), "::MAX`], or if any decoded value is negative, this codec will return an error [`DataResult`].")]
        pub static $name: $unsigned_codec_type = flat_xmap(
            &$transformed_codec,
            |i| <$unsigned_prim>::try_from(i)
                    .map_or_else(|_| DataResult::error(concat!("Could not fit ", stringify!($signed_prim), " into ", stringify!($unsigned_prim)).to_string()), DataResult::success),
            |u| <$signed_prim>::try_from(*u)
                .map_or_else(|_| DataResult::error(concat!("Could not fit ", stringify!($unsigned_prim), " into ", stringify!($signed_prim)).to_string()), DataResult::success),
        );
    };
}

impl_unsigned_transformer_codec!(UBYTE_CODEC, ByteCodec, UbyteCodec, u8, i8, BYTE_CODEC);
impl_unsigned_transformer_codec!(USHORT_CODEC, ShortCodec, UshortCodec, u16, i16, SHORT_CODEC);
impl_unsigned_transformer_codec!(UINT_CODEC, IntCodec, UintCodec, u32, i32, INT_CODEC);
impl_unsigned_transformer_codec!(ULONG_CODEC, LongCodec, UlongCodec, u64, i64, LONG_CODEC);

// Modifier methods

/// Creates a [`LazyCodec`] with a *function pointer* that returns a new [`Codec`], which will be called on first use.
pub const fn lazy<C: Codec>(f: fn() -> C) -> LazyCodec<C> {
    new_lazy_codec(f)
}

/// Creates a [`ListCodec`] of another [`Codec`] with the provided minimum and maximum size.
pub const fn list<C: Codec>(codec: &'static C, min_size: usize, max_size: usize) -> ListCodec<C> {
    new_list_codec(codec, min_size, max_size)
}

/// Creates a [`ListCodec`] of another [`Codec`] with the provided maximum size.
pub const fn limited_list<C: Codec>(codec: &'static C, max_size: usize) -> ListCodec<C> {
    new_list_codec(codec, 0, max_size)
}

/// Creates a [`ListCodec`] of another [`Codec`], which allows any size.
pub const fn unbounded_list<C: Codec>(codec: &'static C) -> ListCodec<C> {
    new_list_codec(codec, 0, usize::MAX)
}

/// Helper macro to generate the shorthand types and functions of the transformer [`Codec`] methods.
macro_rules! make_codec_transformation_function {
    ($name:ident, $short_type:ident, $encoder_type:ident, $decoder_type:ident, $encoder_func:ident, $decoder_func:ident, $to_func_result:ty, $from_func_result:ty, $a_equivalency:literal, $s_equivalency:literal) => {
        pub type $short_type<S, C> = ComposedCodec<$encoder_type<S, C>, $decoder_type<S, C>>;

        #[doc = "Transforms a [`Codec`] of type `A` to another [`Codec`] of type `S`."]
        ///
        /// - `to` is the function called on `A` after decoding to convert it to `S`.
        /// - `from` is the function called on `S` before encoding to convert it to `A`.
        ///
        /// Use this if:
        #[doc = concat!("- `A` is **", $a_equivalency, "** to `S`.")]
        #[doc = concat!("- `S` is **", $s_equivalency, "** to `A`.")]
        #[doc = ""]
        #[doc = "A type `A` is *fully equivalent* to `B` if *A can always successfully be converted to B*."]
        pub const fn $name<A, C: Codec<Value = A>, S>(codec: &'static C, to: fn(A) -> $to_func_result, from: fn(&S) -> $from_func_result) -> $short_type<S, C> {
            ComposedCodec {
                encoder: $encoder_func(codec, from),
                decoder: $decoder_func(codec, to)
            }
        }
    };
}

// Transformer functions

make_codec_transformation_function!(
    xmap,
    XmapCodec,
    ComappedEncoderImpl,
    MappedDecoderImpl,
    comap,
    map,
    S,
    A,
    "equivalent",
    "equivalent"
);

make_codec_transformation_function!(
    comap_flat_map,
    ComapFlatMapCodec,
    ComappedEncoderImpl,
    FlatMappedDecoderImpl,
    comap,
    flat_map,
    DataResult<S>,
    A,
    "equivalent",
    "partially equivalent"
);

make_codec_transformation_function!(
    flat_map_comap,
    FlatMapComapCodec,
    FlatComappedEncoderImpl,
    MappedDecoderImpl,
    flat_comap,
    map,
    S,
    DataResult<A>,
    "partially equivalent",
    "equivalent"
);

make_codec_transformation_function!(
    flat_xmap,
    FlatXmapCodec,
    FlatComappedEncoderImpl,
    FlatMappedDecoderImpl,
    flat_comap,
    flat_map,
    DataResult<S>,
    DataResult<A>,
    "partially equivalent",
    "partially equivalent"
);

/// Returns a transformer codec that validates a value before encoding and after decoding by calling a function,
/// which provides a [`DataResult`] depending on that value's validity.
///
/// `validator` is a function that takes the pointer of a value and returns a [`Result`].
/// - If the returned result is an [`Ok`], the codec works as normal.
/// - Otherwise, it always returns a non-result with the message [`String`].
pub const fn validate<C: Codec>(
    codec: &'static C,
    validator: fn(&C::Value) -> Result<(), String>,
) -> ValidatedCodec<C> {
    new_validated_codec(codec, validator)
}

// Range codec functions

macro_rules! make_codec_range_function {
    ($func_name:ident, $shorthand_name:ident, $ty:ty, $codec:ident, $singleton_codec:ident, $java_type:ident) => {
        pub type $shorthand_name = RangeCodec<$codec>;

        #[doc = concat!("Returns a version of [`", stringify!($singleton_codec), "`] for `", stringify!($ty), "`s (or `", stringify!($java_type), "`s in Java) constrained to a minimum *(inclusive)* and maximum *(inclusive)* value.")]
        pub const fn $func_name(min: $ty, max: $ty) -> $shorthand_name {
            new_range_codec(&$singleton_codec, min, max)
        }
    };
}

make_codec_range_function!(int_range, IntRangeCodec, i32, IntCodec, INT_CODEC, int);
make_codec_range_function!(
    float_range,
    FloatRangeCodec,
    f32,
    FloatCodec,
    FLOAT_CODEC,
    float
);
make_codec_range_function!(
    double_range,
    DoubleRangeCodec,
    f64,
    DoubleCodec,
    DOUBLE_CODEC,
    double
);

// Map codec functions

/// Creates a [`SimpleMapCodec`] with the provided key codec, value (element) codec and the possible key values.
pub const fn simple_map<K: Codec, V: Codec, Key: Keyable>(
    key_codec: &'static K,
    element_codec: &'static V,
    keyable: Key,
) -> SimpleMapCodec<K, V, Key>
where
    <K as HasValue>::Value: Display + Eq + Hash,
{
    new_simple_map_codec(key_codec, element_codec, keyable)
}

/// Creates an [`UnboundedMapCodec`] with the provided key and value (element) codec.
pub const fn unbounded_map<K: Codec, V: Codec>(
    key_codec: &'static K,
    element_codec: &'static V,
) -> UnboundedMapCodec<K, V>
where
    <K as HasValue>::Value: Display + Eq + Hash,
{
    new_unbounded_map_codec(key_codec, element_codec)
}

// Struct codec functions

/// Creates a structure [`Codec`]. This macro supports up to *16* [`Field`]s.
///
/// Struct codec types are usually pretty large. To combat this, use `pub type ... = ...` to
/// only store the complicated type once and never use it again. Rust can easily infer the type
/// for you after you define your codec.
///
/// # Example
/// ```rust
/// use pumpkin_datafixerupper::serialization::codec::*;
/// use pumpkin_datafixerupper::serialization::codecs::primitive::*;
/// use pumpkin_datafixerupper::serialization::struct_codecs::*;
/// use pumpkin_datafixerupper::struct_codec;
///
/// // An example struct to make a codec for.
/// pub struct Person {
///     name: String,
///     age: u32
/// }
///
/// // Type to avoid writing this struct codec's type again.
/// pub type PersonCodec = StructCodec2<Person, FieldMapCodec<StringCodec>, FieldMapCodec<UintCodec>>;
///
/// // The actual codec.
/// pub static PERSON_CODEC: PersonCodec = struct_codec!(
///      for_getter(field(&STRING_CODEC, "name"), |person: &Person| &person.name),
///      for_getter(field(&UINT_CODEC, "age"), |person: &Person| &person.age),
///      |name, age| Person {name, age}
///  );
/// ```
#[macro_export]
macro_rules! struct_codec {
    ($f1:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codecs::struct_1($f1, $f)
    };
    ($f1:expr, $f2:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codecs::struct_2($f1, $f2, $f)
    };
    ($f1:expr, $f2:expr, $f3:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codecs::struct_3($f1, $f2, $f3, $f)
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codecs::struct_4($f1, $f2, $f3, $f4, $f)
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codecs::struct_5($f1, $f2, $f3, $f4, $f5, $f)
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f6:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codecs::struct_6($f1, $f2, $f3, $f4, $f5, $f6, $f)
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f6:expr, $f7:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codecs::struct_7($f1, $f2, $f3, $f4, $f5, $f6, $f7, $f)
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f6:expr, $f7:expr, $f8:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codecs::struct_8($f1, $f2, $f3, $f4, $f5, $f6, $f7, $f8, $f)
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f6:expr, $f7:expr, $f8:expr, $f9:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codecs::struct_9(
            $f1, $f2, $f3, $f4, $f5, $f6, $f7, $f8, $f9, $f,
        )
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f6:expr, $f7:expr, $f8:expr, $f9:expr, $f10:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codecs::struct_10(
            $f1, $f2, $f3, $f4, $f5, $f6, $f7, $f8, $f9, $f10, $f,
        )
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f6:expr, $f7:expr, $f8:expr, $f9:expr, $f10:expr, $f11:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codecs::struct_11(
            $f1, $f2, $f3, $f4, $f5, $f6, $f7, $f8, $f9, $f10, $f11, $f,
        )
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f6:expr, $f7:expr, $f8:expr, $f9:expr, $f10:expr, $f11:expr, $f12:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codecs::struct_12(
            $f1, $f2, $f3, $f4, $f5, $f6, $f7, $f8, $f9, $f10, $f11, $f12, $f,
        )
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f6:expr, $f7:expr, $f8:expr, $f9:expr, $f10:expr, $f11:expr, $f12:expr, $f13:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codecs::struct_13(
            $f1, $f2, $f3, $f4, $f5, $f6, $f7, $f8, $f9, $f10, $f11, $f12, $f13, $f,
        )
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f6:expr, $f7:expr, $f8:expr, $f9:expr, $f10:expr, $f11:expr, $f12:expr, $f13:expr, $f14:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codecs::struct_14(
            $f1, $f2, $f3, $f4, $f5, $f6, $f7, $f8, $f9, $f10, $f11, $f12, $f13, $f14, $f,
        )
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f6:expr, $f7:expr, $f8:expr, $f9:expr, $f10:expr, $f11:expr, $f12:expr, $f13:expr, $f14:expr, $f15:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codecs::struct_15(
            $f1, $f2, $f3, $f4, $f5, $f6, $f7, $f8, $f9, $f10, $f11, $f12, $f13, $f14, $f15, $f,
        )
    };
    ($f1:expr, $f2:expr, $f3:expr, $f4:expr, $f5:expr, $f6:expr, $f7:expr, $f8:expr, $f9:expr, $f10:expr, $f11:expr, $f12:expr, $f13:expr, $f14:expr, $f15:expr, $f16:expr, $f:expr $(,)?) => {
        $crate::serialization::struct_codecs::struct_16(
            $f1, $f2, $f3, $f4, $f5, $f6, $f7, $f8, $f9, $f10, $f11, $f12, $f13, $f14, $f15, $f16,
            $f,
        )
    };
}

// Field functions

/// A type of [`MapCodec`] to encode/decode for a single field of a map with the help of a [`Codec`].
pub type FieldMapCodec<C> = ComposedMapCodec<
    FieldEncoder<<C as HasValue>::Value, C>,
    FieldDecoder<<C as HasValue>::Value, C>,
>;

/// Creates a [`MapCodec`] for a field which relies on the provided [`Codec`] for serialization/deserialization.
pub const fn field<C: Codec>(codec: &'static C, name: &'static str) -> FieldMapCodec<C> {
    ComposedMapCodec {
        encoder: encoder_field(name, codec),
        decoder: decoder_field(name, codec),
    }
}

/// Creates a [`MapCodec`] for an optional field which relies on the provided [`Codec`] for serialization/deserialization.
///
/// Since this `MapCodec` has no 'default value', this is equivalent to encoding an [`Option`].
/// The returned `MapCodec` is also *not lenient*, meaning that it will not give a complete (successful) result
/// if the decoded field value is an error [`DataResult`] (partial or no result). Most of the time, you will
/// want a *non-lenient* field.
pub const fn optional_field<C: Codec>(
    codec: &'static C,
    name: &'static str,
) -> OptionalFieldMapCodec<C> {
    new_optional_field_map_codec(codec, name, false)
}

/// Creates a [`MapCodec`] for an optional field which relies on the provided [`Codec`] for serialization/deserialization.
///
/// Since this `MapCodec` has no 'default value', this is equivalent to encoding an [`Option`].
/// The returned `MapCodec` is also *lenient*, meaning that it will still give a complete (successful) result
/// if the decoded field value is an error [`DataResult`] (partial or no result). Most of the time, you will
/// want a *non-lenient* field.
pub const fn lenient_optional_field<C: Codec>(
    codec: &'static C,
    name: &'static str,
) -> OptionalFieldMapCodec<C> {
    new_optional_field_map_codec(codec, name, false)
}

pub type DefaultedFieldCodec<C> =
    DefaultValueProviderMapCodec<<C as HasValue>::Value, OptionalFieldMapCodec<C>>;

/// Creates a [`MapCodec`] for an optional field which relies on the provided [`Codec`] for serialization/deserialization, along with a default value factory.
///
/// The factory provided is used for equality checks and for creating a new default value
/// for when no value is found.
///
/// The returned `MapCodec` is also *not lenient*, meaning that it will not give a complete (successful) result
/// if the decoded field value is an error [`DataResult`] (partial or no result). Most of the time, you will
/// want a *non-lenient* field.
pub const fn optional_field_with_default<C: Codec>(
    codec: &'static C,
    name: &'static str,
    factory: fn() -> C::Value,
) -> DefaultedFieldCodec<C>
where
    <C as HasValue>::Value: PartialEq + Clone,
{
    new_default_value_provider_map_codec(new_optional_field_map_codec(codec, name, false), factory)
}

/// Creates a [`MapCodec`] for an optional field which relies on the provided [`Codec`] for serialization/deserialization, along with a default value factory.
///
/// The factory provided is used for equality checks and for creating a new default value
/// for when no value is found.
///
/// The returned `MapCodec` is also *lenient*, meaning that it will still give a complete (successful) result
/// if the decoded field value is an error [`DataResult`] (partial or no result). Most of the time, you will
/// want a *non-lenient* field.
pub const fn lenient_optional_field_with_default<C: Codec>(
    codec: &'static C,
    name: &'static str,
    factory: fn() -> C::Value,
) -> DefaultedFieldCodec<C>
where
    <C as HasValue>::Value: PartialEq + Clone,
{
    new_default_value_provider_map_codec(new_optional_field_map_codec(codec, name, true), factory)
}

// Assertion functions

/// Asserts that the decoding of some value by a [`DynamicOps`] via a [`Codec`] is a success/error.
/// # Example
/// ```
/// # use pumpkin_datafixerupper::assert_decode;
/// # use serde_json::json;
/// # use pumpkin_datafixerupper::serialization::json_ops;
/// # use pumpkin_datafixerupper::serialization::codec;
/// # use crate::pumpkin_datafixerupper::serialization::coders::Decoder;
///
/// assert_decode!(codec::INT_CODEC, json!(2), &json_ops::INSTANCE, is_success);
/// assert_decode!(codec::STRING_CODEC, json!("hello"), &json_ops::INSTANCE, is_success);
/// assert_decode!(codec::FLOAT_CODEC, json!(true), &json_ops::INSTANCE, is_error);
/// ```
#[macro_export]
macro_rules! assert_decode {
    ($codec:expr, $value:expr, $ops:expr, $assertion:ident) => {{
        assert!($codec.decode($value, $ops).$assertion());
    }};
}
