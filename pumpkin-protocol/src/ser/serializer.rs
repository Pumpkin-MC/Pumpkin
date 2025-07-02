use std::fmt::Display;

use serde::{
    Serialize,
    ser::{self, Impossible},
};

use super::{NO_PREFIX_MARKER, NetworkWriteExt, Write, WritingError};

pub struct Serializer<W: Write> {
    pub write: W,
}

impl<W: Write> Serializer<W> {
    pub fn new(w: W) -> Self {
        Self { write: w }
    }
}

impl ser::Error for WritingError {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}

/// This serializer just writes a sequence without a varint prefix and defers the rest of the
/// serialization to the wrapped serializer
struct NonPrefixedSeqSerializer<'a, W: Write> {
    wrapped: &'a mut Serializer<W>,
}

macro_rules! create_fail_method {
    ($method: ident, $ty: ty) => {
        fn $method(self, _v: $ty) -> Result<Self::Ok, Self::Error> {
            Err(WritingError::Serde(format!(
                "Expected a sequence, but found {}!",
                stringify!($ty)
            )))
        }
    };
}

impl<W: Write> ser::SerializeSeq for NonPrefixedSeqSerializer<'_, W> {
    type Ok = ();
    type Error = WritingError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut *self.wrapped).map(|_| ())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W: Write> ser::Serializer for NonPrefixedSeqSerializer<'_, W> {
    type Ok = ();
    type Error = WritingError;

    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeSeq = Self;

    create_fail_method!(serialize_bool, bool);
    create_fail_method!(serialize_bytes, &[u8]);
    create_fail_method!(serialize_char, char);
    create_fail_method!(serialize_f32, f32);
    create_fail_method!(serialize_f64, f64);
    create_fail_method!(serialize_i8, i8);
    create_fail_method!(serialize_i16, i16);
    create_fail_method!(serialize_i32, i32);
    create_fail_method!(serialize_i64, i64);
    create_fail_method!(serialize_u8, u8);
    create_fail_method!(serialize_u16, u16);
    create_fail_method!(serialize_u32, u32);
    create_fail_method!(serialize_u64, u64);
    create_fail_method!(serialize_str, &str);

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(WritingError::Serde(
            "Expected a sequence but found a map!".into(),
        ))
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(WritingError::Serde(format!(
            "Expected a sequence but found a newtype struct {name}!"
        )))
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(WritingError::Serde(format!(
            "Expected a sequence but found a newtype variant {name}!"
        )))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.wrapped.serialize_none()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.wrapped.serialize_bool(true)?;
        value.serialize(self)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(WritingError::Serde(format!(
            "Expected a sequence but found a struct {name}!"
        )))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(WritingError::Serde(format!(
            "Expected a sequence but found a struct variant {name}!"
        )))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(WritingError::Serde(
            "Expected a sequence but found a tuple!".into(),
        ))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(WritingError::Serde(format!(
            "Expected a sequence but found a tuple struct {name}!"
        )))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(WritingError::Serde(format!(
            "Expected a sequence but found a tuple variant {name}!"
        )))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(WritingError::Serde(
            "Expected a sequence but found a unit!".into(),
        ))
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(WritingError::Serde(format!(
            "Expected a sequence but found a unit struct {name}!"
        )))
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(WritingError::Serde(format!(
            "Expected a sequence but found a unit variant {name}!"
        )))
    }
}

// General notes on the serializer:
//
// Primitives are written as-is
// Strings automatically prepend a VarInt
// Enums are written as a VarInt of the index
// Structs are ignored
// Iterables' values are written in order, but NO information (e.g. size) about the
// iterable itself is written (list sizes should be a separate field)
impl<W: Write> ser::Serializer for &mut Serializer<W> {
    type Ok = ();
    type Error = WritingError;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.write.write_bool(v)
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.write.write_slice(v)
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.write.write_u32_be(v as u32)
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.write.write_f32_be(v)
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.write.write_f64_be(v)
    }
    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        self.write
            .write_all(&v.to_be_bytes())
            .map_err(WritingError::IoError)
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.write.write_i16_be(v)
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.write.write_i32_be(v)
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.write.write_i64_be(v)
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.write.write_i8(v)
    }
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let Some(len) = len else {
            return Err(WritingError::Serde("Maps must have a known length".into()));
        };

        self.write.write_var_int(&len.try_into().map_err(|_| {
            WritingError::Message(format!("{len} isn't representable as a VarInt"))
        })?)?;

        Ok(self)
    }
    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        // TODO: This is super sketchy... is there a way to do it better? Can we choose what
        // serializer to use on a struct somehow from within the struct?
        if name == "TextComponent" {
            let mut nbt_serializer =
                pumpkin_nbt::serializer::Serializer::new(&mut self.write, None);
            value.serialize(&mut nbt_serializer).map_err(|err| {
                WritingError::Serde(format!("Failed to serialize TextComponent NBT: {err}"))
            })
        } else if name == NO_PREFIX_MARKER {
            value.serialize(NonPrefixedSeqSerializer { wrapped: self })
        } else {
            value.serialize(self)
        }
    }
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.write
            .write_var_int(&variant_index.try_into().map_err(|_| {
                WritingError::Message(format!("{variant_index} isn't representable as a VarInt"))
            })?)?;
        value.serialize(self)
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.write.write_bool(false)
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let Some(len) = len else {
            return Err(WritingError::Serde(
                "Sequences must have a known length".into(),
            ));
        };

        self.write.write_var_int(&len.try_into().map_err(|_| {
            WritingError::Message(format!("{len} isn't representable as a VarInt"))
        })?)?;

        Ok(self)
    }
    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.write.write_bool(true)?;
        value.serialize(self)
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.write.write_string(v)
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        // Serialize ENUM index as varint
        self.write
            .write_var_int(&variant_index.try_into().map_err(|_| {
                WritingError::Message(format!("{variant_index} isn't representable as a VarInt"))
            })?)?;
        Ok(self)
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        // Serialize ENUM index as varint
        self.write
            .write_var_int(&variant_index.try_into().map_err(|_| {
                WritingError::Message(format!("{variant_index} isn't representable as a VarInt"))
            })?)?;
        Ok(self)
    }
    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        self.write
            .write_all(&v.to_be_bytes())
            .map_err(WritingError::IoError)
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.write.write_u16_be(v)
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.write.write_u32_be(v)
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.write.write_u64_be(v)
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.write.write_u8(v)
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        // For ENUMs, only write enum index as varint
        self.write
            .write_var_int(&variant_index.try_into().map_err(|_| {
                WritingError::Message(format!("{variant_index} isn't representable as a VarInt"))
            })?)
    }
    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<W: Write> ser::SerializeSeq for &mut Serializer<W> {
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = WritingError;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    // Close the sequence.
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<W: Write> ser::SerializeTuple for &mut Serializer<W> {
    type Ok = ();
    type Error = WritingError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

// Same thing but for tuple structs.
impl<W: Write> ser::SerializeTupleStruct for &mut Serializer<W> {
    type Ok = ();
    type Error = WritingError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

// Tuple variants are a little different. Refer back to the
// `serialize_tuple_variant` method above:
//
//    self.write += "{";
//    variant.serialize(&mut *self)?;
//    self.write += ":[";
//
// So the `end` method in this impl is responsible for closing both the `]` and
// the `}`.
impl<W: Write> ser::SerializeTupleVariant for &mut Serializer<W> {
    type Ok = ();
    type Error = WritingError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

// Some `Serialize` types are not able to hold a key and value in memory at the
// same time, so `SerializeMap` implementations are required to support
// `serialize_key` and `serialize_value` individually.
//
// There is a third optional method on the `SerializeMap` trait. The
// `serialize_entry` method allows serializers to optimize for the case where
// key and value are both available simultaneously. In JSON it doesn't make a
// difference, so the default behavior for `serialize_entry` is fine.
impl<W: Write> ser::SerializeMap for &mut Serializer<W> {
    type Ok = ();
    type Error = WritingError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<W: Write> ser::SerializeStruct for &mut Serializer<W> {
    type Ok = ();
    type Error = WritingError;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
        let _ = key;
        Ok(())
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<W: Write> ser::SerializeStructVariant for &mut Serializer<W> {
    type Ok = ();
    type Error = WritingError;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
