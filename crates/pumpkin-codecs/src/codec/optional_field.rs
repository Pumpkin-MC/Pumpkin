use crate::codec::FieldEncode;
use crate::map_like::MapLike;
use crate::struct_builder::StructBuilder;
use crate::{DataResult, Decode, DynamicOps, Encode};

/// A trait for something which can be added to a [`MapLike`] as an optional field with a provided name.
pub trait OptionalFieldEncode {
    /// Encodes this value to a map by adding an optional field, whose:
    /// - key is the field's `name`.
    /// - value is the encoded value represented by the provided [`DynamicOps`].
    fn encode_optional_field<O: DynamicOps, B: StructBuilder<Value = O::Value>>(
        &self,
        name: &'static str,
        ops: &'static O,
        prefix: B,
    ) -> B;
}

impl<T> OptionalFieldEncode for Option<T>
where
    T: Encode,
{
    fn encode_optional_field<O: DynamicOps, B: StructBuilder<Value = O::Value>>(
        &self,
        name: &'static str,
        ops: &'static O,
        prefix: B,
    ) -> B {
        if let Some(value) = self {
            value.encode_field(name, ops, prefix)
        } else {
            prefix
        }
    }
}

/// A trait to decode an optional field of a [`MapLike`] into a
/// value of the implementing type.
///
/// There is no `OptionalFieldEncode` variant of this trait; just
/// use [`FieldEncode::encode_field`] for encoding an optional field.
pub trait OptionalFieldDecode: Sized {
    /// Decodes an optional field from a map, similar to [`FieldDecode::decode_field`].
    ///
    /// However, this method has an extra `lenient` parameter. If it is `true`, errors
    /// while decoding a `Some` option will not occur, and a `None` will be decoded instead.
    fn decode_optional_field<O: DynamicOps>(
        name: &'static str,
        input: &impl MapLike<Value = O::Value>,
        ops: &'static impl DynamicOps<Value = O::Value>,
        lenient: bool,
    ) -> DataResult<Self>;
}

impl<T> OptionalFieldDecode for Option<T>
where
    T: Decode,
{
    fn decode_optional_field<O: DynamicOps>(
        name: &'static str,
        input: &impl MapLike<Value = O::Value>,
        ops: &'static impl DynamicOps<Value = O::Value>,
        lenient: bool,
    ) -> DataResult<Self> {
        input.get_str(name).map_or_else(
            || DataResult::new_success(None),
            |value| {
                let result = T::parse(value.clone(), ops);
                if result.is_error() && lenient {
                    DataResult::new_success(None)
                } else {
                    result.map(Some)
                }
            },
        )
    }
}
