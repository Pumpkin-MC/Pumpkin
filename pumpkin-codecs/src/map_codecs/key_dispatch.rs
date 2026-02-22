use crate::impl_compressor;
use crate::HasValue;
use crate::data_result::DataResult;
use crate::dynamic_ops::DynamicOps;
use crate::key_compressor::KeyCompressor;
use crate::keyable::Keyable;
use crate::map_codec::MapCodec;
use crate::map_coders::{CompressorHolder, MapDecoder, MapEncoder};
use crate::map_like::MapLike;
use crate::struct_builder::StructBuilder;
use std::fmt::Display;
use std::marker::PhantomData;

/// The key used to hold the map containing specific fields to encode for a variant
/// when maps are compressed.
static COMPRESSED_VALUE_KEY: &str = "value";

/// A trait for which a [`KeyDispatchMapCodec`] can be made. In most cases,
/// using the [`crate::impl_key_dispatchable!`] macro is enough for your needs.
///
/// If you want to implement this trait manually, it requires implementing 3 functions,
/// which do something depending on a variant type:
/// - [`KeyDispatchable::decode`]
/// - [`KeyDispatchable::map_encode`]
/// - [`KeyDispatchable::map_decode`]
pub trait KeyDispatchable: Sized {
    type Key;

    /// Decodes a value to this type using a [`Decoder`] implementation of a [`MapDecoder`].
    fn decode<T: Display + PartialEq + Clone>(
        key: Self::Key,
        input: T,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<Self>;

    /// Encodes a value of this type using a [`MapEncoder`].
    fn map_encode<T: Display + PartialEq + Clone, B: StructBuilder<Value = T>>(
        &self,
        input: &Self,
        ops: &'static impl DynamicOps<Value = T>,
        prefix: B,
    ) -> B;

    /// Decodes a value to this type using a [`MapDecoder`].
    fn map_decode<T: Display + PartialEq + Clone>(
        key: Self::Key,
        input: &impl MapLike<Value = T>,
        ops: &'static impl DynamicOps<Value = T>,
    ) -> DataResult<Self>;
}

/// Convenience macro to only implement a getter for some variant.
///
/// # Example
/// ```rust
/// # use pumpkin_codecs::{struct_map_codec, impl_getter_variant, impl_key_dispatchable};
/// # use pumpkin_codecs::codec::{field, FieldMapCodec, INT_CODEC, STRING_CODEC};
/// # use pumpkin_codecs::codecs::primitive::*;
/// # use pumpkin_codecs::coders::{Encoder, Decoder};
/// # use pumpkin_codecs::map_codec::for_getter;
/// # use pumpkin_codecs::map_coders::{MapEncoder, MapDecoder};
/// # use pumpkin_codecs::struct_codecs::StructMapCodec1;
/// #
/// # use pumpkin_codecs::map_codecs::key_dispatch::*;
///
/// pub enum Example {
///     A(String),
///     B { field: i32 }
/// }
///
/// /// A [`MapCodec`] for specific fields of `Example::A`.
/// pub static A_MAP_CODEC: StructMapCodec1<Example, FieldMapCodec<StringCodec>> = struct_map_codec!(
///     for_getter(field(&STRING_CODEC, "a_field"), impl_getter_variant!(Example::A(x) => x)),
///     Example::A
/// );
///
/// /// A [`MapCodec`] for specific fields of `Example::B`.
/// pub static B_MAP_CODEC: StructMapCodec1<Example, FieldMapCodec<IntCodec>> = struct_map_codec!(
///     for_getter(field(&INT_CODEC, "b_field"),  impl_getter_variant!(Example::B {field: x} => x)),
///     |i| Example::B { field: i }
/// );
/// ```
#[macro_export]
macro_rules! impl_getter_variant {
    ($pattern:pat => $result:expr) => {
        |ty|
            if let $pattern = ty {
                $result
            } else {
                unreachable!("Tried to encode a value of an enum type using a MapCodec of a different enum type")
            }
    };
}

/// A macro to generate an implementation of [`KeyDispatchable`].
///
/// This macro has two ways of using it:
/// - Using a *String* to differentiate between variants.
/// - Using an *enum* to differentiate between variants.
///   The enum **must** implement `Display` for errors.
///
/// Place this in an `impl KeyDispatchable for ...` block.
///
/// In the macro, you place a branch for each variant. Here's an example:
/// ```txt
/// (Self::A(..), "a") => A_MAP_CODEC
/// ```
/// Here, `Self::A(...)` is a pattern and `"a"` is the differentiator pattern (a string slice here).
/// `A_MAP_CODEC` is the map codec that will be used for encoding/decoding the specific fields of `A`.
///
/// You don't have to add a branch for every variant, but any left variant `MapCodec`s will be considered `todo!()`,
/// so if *map* encoding is attempted with an unimplemented variant, it will **panic**.
///
/// # Examples
///
/// ```rust
/// # use pumpkin_codecs::{struct_map_codec, impl_getter_variant, impl_key_dispatchable};
/// # use pumpkin_codecs::codec::{field, FieldMapCodec, INT_CODEC, STRING_CODEC};
/// # use pumpkin_codecs::codecs::primitive::*;
/// # use pumpkin_codecs::coders::{Encoder, Decoder};
/// # use pumpkin_codecs::map_codec::for_getter;
/// # use pumpkin_codecs::map_coders::{MapEncoder, MapDecoder};
/// # use pumpkin_codecs::struct_codecs::StructMapCodec1;
///
/// use pumpkin_codecs::map_codecs::key_dispatch::*;
///
/// /// Our example `KeyDispatchable`.
/// pub enum Example {
///     A(String),
///     B { field: i32 }
/// }
///
/// // `MapCodec`s for specific variants will probably not be used anywhere else, so it's
/// // fine to keep their types as is without using `pub type ...`.
///
/// // pub static A_MAP_CODEC: StructMapCodec1<Example, FieldMapCodec<StringCodec>> = ...
/// // pub static B_MAP_CODEC: StructMapCodec1<Example, FieldMapCodec<IntCodec>> = ...
/// #
/// # pub static A_MAP_CODEC: StructMapCodec1<Example, FieldMapCodec<StringCodec>> = struct_map_codec!(
/// #     for_getter(field(&STRING_CODEC, "a_field"), impl_getter_variant!(Example::A(x) => x)),
/// #     Example::A
/// # );
/// #
/// # pub static B_MAP_CODEC: StructMapCodec1<Example, FieldMapCodec<IntCodec>> = struct_map_codec!(
/// #     for_getter(field(&INT_CODEC, "b_field"),  impl_getter_variant!(Example::B {field: x} => x)),
/// #     |i| Example::B { field: i }
/// # );
///
/// impl KeyDispatchable for Example {
///     // Our macro.
///     impl_key_dispatchable!(
///         // We use a String to differentiate our enum.
///         string,
///         (Self::A(..), "a") => A_MAP_CODEC,
///         (Self::B {..}, "b") => B_MAP_CODEC,
///     );
/// }
///
/// ```
///
/// An enum implementation would look something like this:
///
/// ```rust
/// # use pumpkin_codecs::{struct_map_codec, impl_getter_variant, impl_key_dispatchable};
/// # use pumpkin_codecs::codec::{field, FieldMapCodec, INT_CODEC, STRING_CODEC};
/// # use pumpkin_codecs::codecs::primitive::*;
/// # use pumpkin_codecs::coders::{Encoder, Decoder};
/// # use pumpkin_codecs::map_codec::for_getter;
/// # use pumpkin_codecs::map_coders::{MapEncoder, MapDecoder};
/// # use pumpkin_codecs::struct_codecs::StructMapCodec1;
/// #
/// # use pumpkin_codecs::map_codecs::key_dispatch::*;
/// #
/// # pub static A_MAP_CODEC: StructMapCodec1<Example, FieldMapCodec<StringCodec>> = struct_map_codec!(
/// #     for_getter(field(&STRING_CODEC, "a_field"), impl_getter_variant!(Example::A(x) => x)),
/// #     Example::A
/// # );
/// #
/// # pub static B_MAP_CODEC: StructMapCodec1<Example, FieldMapCodec<IntCodec>> = struct_map_codec!(
/// #     for_getter(field(&INT_CODEC, "b_field"),  impl_getter_variant!(Example::B {field: x} => x)),
/// #     |i| Example::B { field: i }
/// # );
/// use std::fmt;
///
/// /// The enum for which we want to make serialization possible.
/// pub enum Example {
///     A(String),
///     B { field: i32 }
/// }
///
/// /// Our differentiator enum.
/// #[derive(Debug, PartialEq, Eq, Copy, Clone)]
/// pub enum ExampleType {
///     A,
///     B
/// }
///
/// // `Display` implementation for our differentiator enum.
/// impl fmt::Display for ExampleType {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         // We just delegate to the `Debug` implementation.
///         write!(f, "{:?}", self)
///     }
/// }
///
/// impl KeyDispatchable for Example {
///     // Our macro.
///     impl_key_dispatchable!(
///         // We use a String to differentiate our enum.
///         enum ExampleType,
///         (Self::A(..), ExampleType::A) => A_MAP_CODEC,
///         (Self::B {..}, ExampleType::B) => B_MAP_CODEC,
///     );
/// }
/// ```
#[macro_export]
macro_rules! impl_key_dispatchable {
    (
        @internal,

        $(
            ( $variant:pat, $matched:pat ) => $map_codec:ident
        ),+
    ) => {
        // Used to silence the warning for a complete `KeyDispatchable` implementation.
        #[allow(unreachable_patterns)]
        fn map_encode<T: std::fmt::Display + PartialEq + Clone, B: $crate::struct_builder::StructBuilder<Value=T>>(&self, input: &Self, ops: &'static impl $crate::dynamic_ops::DynamicOps<Value=T>, prefix: B) -> B {
            match self {
                $(
                    $variant => $map_codec.encode(input, ops, prefix),
                )+

                _ => todo!("Encode not implemented yet"),
            }
        }
    };

    // For `KeyDispatchable`s that differentiate using a string.
    (
        string,

        // (Enum::FOO, "foo") => FOO_MAP_CODEC
        $(
            ( $variant:pat, $matched:pat ) => $map_codec:ident
        ),+

        $(,)?
    ) => {
        type Key = String;

        fn decode<T: std::fmt::Display + PartialEq + Clone>(key: Self::Key, input: T, ops: &'static impl $crate::dynamic_ops::DynamicOps<Value=T>) -> $crate::data_result::DataResult<Self> {
            match key.as_str() {
                $(
                    $matched => $crate::map_coders::new_map_decoder_decoder(&$map_codec).parse(input, ops),
                )+

                &_ => $crate::data_result::DataResult::error(format!("Invalid differentiator value {key}")),
            }
        }

        fn map_decode<T: std::fmt::Display + PartialEq + Clone>(key: Self::Key, input: &impl $crate::map_like::MapLike<Value=T>, ops: &'static impl $crate::dynamic_ops::DynamicOps<Value=T>) -> $crate::data_result::DataResult<Self> {
            match key.as_str() {
                $(
                    $matched => $map_codec.decode(input, ops),
                )+

                &_ => $crate::data_result::DataResult::error(format!("Invalid differentiator value {key}")),
            }
        }

        impl_key_dispatchable!(@internal $(, ( $variant, $matched ) => $map_codec)+);
    };

    // For `KeyDispatchable`s that differentiate using an enum.
    (
        // Differentiator value enum (must be unit)
        enum $e:ty,

        // (Enum::FOO, Key::Foo) => FOO_MAP_CODEC
        $(
            ( $variant:pat, $matched:pat ) => $map_codec:ident
        ),+

        $(,)?
    ) => {
        type Key = $e;

        #[allow(unreachable_patterns)]
        fn decode<T: std::fmt::Display + PartialEq + Clone>(key: Self::Key, input: T, ops: &'static impl $crate::dynamic_ops::DynamicOps<Value=T>) -> $crate::data_result::DataResult<Self> {
            match key {
                $( $matched => $crate::map_coders::new_map_decoder_decoder(&$map_codec).parse(input, ops), )+

                _ => $crate::data_result::DataResult::error(format!("Invalid differentiator value {key}")),
            }
        }

        #[allow(unreachable_patterns)]
        fn map_decode<T: std::fmt::Display + PartialEq + Clone>(key: Self::Key, input: &impl $crate::map_like::MapLike<Value=T>, ops: &'static impl $crate::dynamic_ops::DynamicOps<Value=T>) -> $crate::data_result::DataResult<Self> {
            match key {
                $( $matched => $map_codec.decode(input, ops), )+

                _ => $crate::data_result::DataResult::error(format!("Invalid differentiator value {key}")),
            }
        }

        impl_key_dispatchable!(@internal $(, ( $variant, $matched ) => $map_codec)+);
    };
}

/// A type of [`MapCodec`] to handle encoding a struct-like type whose fields
/// can differ by variant. A very common example of this would be an *enum*.
///
/// This is only implementable for a type which implements [`KeyDispatchable`].
///
/// - `T` is the type to encode to/decode from.
/// - `M` is the `MapCodec` of the key value to differentiate between variants.
///   A common example would be `field(&STRING_CODEC, "type")`.
pub struct KeyDispatchMapCodec<T: KeyDispatchable, M: MapCodec<Value = T::Key> + 'static> {
    key_codec: M,
    phantom: PhantomData<T>,
}

impl<T: KeyDispatchable, M: MapCodec<Value = T::Key>> HasValue for KeyDispatchMapCodec<T, M> {
    type Value = T;
}

impl<T: KeyDispatchable, M: MapCodec<Value = T::Key>> Keyable for KeyDispatchMapCodec<T, M> {
    fn keys(&self) -> Vec<String> {
        let mut vec = self.key_codec.keys();
        vec.push(COMPRESSED_VALUE_KEY.to_string());
        vec
    }
}

impl<T: KeyDispatchable, M: MapCodec<Value = T::Key>> CompressorHolder
    for KeyDispatchMapCodec<T, M>
{
    impl_compressor!();
}

impl<T: KeyDispatchable, M: MapCodec<Value = T::Key>> MapEncoder for KeyDispatchMapCodec<T, M> {
    fn encode<U: Display + PartialEq + Clone, B: StructBuilder<Value = U>>(
        &self,
        input: &Self::Value,
        ops: &'static impl DynamicOps<Value = U>,
        prefix: B,
    ) -> B {
        input.map_encode(input, ops, prefix)
    }
}

impl<T: KeyDispatchable, M: MapCodec<Value = T::Key>> MapDecoder for KeyDispatchMapCodec<T, M> {
    fn decode<U: Display + PartialEq + Clone>(
        &self,
        input: &impl MapLike<Value = U>,
        ops: &'static impl DynamicOps<Value = U>,
    ) -> DataResult<Self::Value> {
        self.key_codec.decode(input, ops).flat_map(|key| {
            if ops.compress_maps() {
                // If the ops we're using compresses maps, we decode
                // the data in the individual "value" key using a normal `Codec`.
                input.get_str(COMPRESSED_VALUE_KEY).map_or_else(
                    || DataResult::error("Input doesn't have a 'value' key".to_string()),
                    |v| T::decode(key, v.clone(), ops),
                )
            } else {
                // We decode the fields in our map directly.
                T::map_decode(key, input, ops)
            }
        })
    }
}

pub(crate) const fn new_key_dispatch_map_codec<T: KeyDispatchable, C: MapCodec<Value = T::Key>>(key_codec: C) -> KeyDispatchMapCodec<T, C> {
    KeyDispatchMapCodec {
        key_codec,
        phantom: PhantomData,
    }
}
