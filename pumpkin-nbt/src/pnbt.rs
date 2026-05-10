use crate::{Error, NBT_ARRAY_TAG, NBT_BYTE_ARRAY_TAG, NBT_INT_ARRAY_TAG, NBT_LONG_ARRAY_TAG};
use serde::{
    Deserialize, Serialize,
    de::{self, MapAccess, SeqAccess, Visitor},
    ser,
};

/// Serializes struct to PNBT (Pumpkin NBT) format.
/// PNBT is a high-performance, positional binary format.
#[inline]
pub fn to_pnbt<T: Serialize>(value: &T) -> Result<Vec<u8>, Error> {
    let mut serializer = Serializer::new();
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

/// Deserializes struct from PNBT format.
#[inline]
pub fn from_pnbt<'a, T: Deserialize<'a>>(input: &'a [u8]) -> Result<T, Error> {
    let mut deserializer = Deserializer::new(input);
    T::deserialize(&mut deserializer)
}

/// `PNbtCompound` is a direct byte-wrapper for building or reading PNBT data.
/// It provides a manual Protobuf-like API without string keys.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PNbtCompound {
    pub data: Vec<u8>,
    pub read_pos: usize,
}

impl PNbtCompound {
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(1024),
            read_pos: 0,
        }
    }

    #[must_use]
    pub const fn from_bytes(data: Vec<u8>) -> Self {
        Self { data, read_pos: 0 }
    }

    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    // --- Writing API ---

    pub fn put_bool(&mut self, v: bool) {
        self.data.push(u8::from(v));
    }

    pub fn put_i8(&mut self, v: i8) {
        self.data.push(v as u8);
    }

    /// Alias for `put_i8`
    pub fn put_byte(&mut self, v: i8) {
        self.put_i8(v);
    }

    pub fn put_u8(&mut self, v: u8) {
        self.data.push(v);
    }

    fn write_varint(&mut self, mut value: u64) {
        loop {
            let byte = (value & 0x7f) as u8;
            value >>= 7;
            if value == 0 {
                self.data.push(byte);
                break;
            }
            self.data.push(byte | 0x80);
        }
    }

    fn write_zigzag(&mut self, value: i64) {
        let encoded = ((value << 1) ^ (value >> 63)) as u64;
        self.write_varint(encoded);
    }

    pub fn put_i16(&mut self, v: i16) {
        self.write_zigzag(i64::from(v));
    }

    /// Alias for `put_i16`
    pub fn put_short(&mut self, v: i16) {
        self.put_i16(v);
    }

    pub fn put_u16(&mut self, v: u16) {
        self.write_varint(u64::from(v));
    }

    pub fn put_i32(&mut self, v: i32) {
        self.write_zigzag(i64::from(v));
    }

    /// Alias for `put_i32`
    pub fn put_int(&mut self, v: i32) {
        self.put_i32(v);
    }

    pub fn put_u32(&mut self, v: u32) {
        self.write_varint(u64::from(v));
    }

    pub fn put_i64(&mut self, v: i64) {
        self.write_zigzag(v);
    }

    /// Alias for `put_i64`
    pub fn put_long(&mut self, v: i64) {
        self.put_i64(v);
    }

    pub fn put_u64(&mut self, v: u64) {
        self.write_varint(v);
    }

    pub fn put_f32(&mut self, v: f32) {
        self.data.extend_from_slice(&v.to_le_bytes());
    }

    /// Alias for `put_f32`
    pub fn put_float(&mut self, v: f32) {
        self.put_f32(v);
    }

    pub fn put_f64(&mut self, v: f64) {
        self.data.extend_from_slice(&v.to_le_bytes());
    }

    /// Alias for `put_f64`
    pub fn put_double(&mut self, v: f64) {
        self.put_f64(v);
    }

    pub fn put_string(&mut self, v: &str) {
        self.write_varint(v.len() as u64);
        self.data.extend_from_slice(v.as_bytes());
    }

    pub fn put_bytes(&mut self, v: &[u8]) {
        self.write_varint(v.len() as u64);
        self.data.extend_from_slice(v);
    }

    pub fn put_uuid(&mut self, v: &uuid::Uuid) {
        self.data.extend_from_slice(v.as_bytes());
    }

    // --- Reading API ---

    fn read_byte(&mut self) -> Result<u8, Error> {
        if self.read_pos >= self.data.len() {
            return Err(Error::SerdeError("EOF".to_string()));
        }
        let b = self.data[self.read_pos];
        self.read_pos += 1;
        Ok(b)
    }

    fn read_varint(&mut self) -> Result<u64, Error> {
        let mut value = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_byte()?;
            value |= ((byte & 0x7f) as u64) << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }
        Ok(value)
    }

    fn read_zigzag(&mut self) -> Result<i64, Error> {
        let value = self.read_varint()?;
        Ok(((value >> 1) as i64) ^ (-((value & 1) as i64)))
    }

    pub fn get_bool(&mut self) -> Result<bool, Error> {
        Ok(self.read_byte()? != 0)
    }

    pub fn get_i8(&mut self) -> Result<i8, Error> {
        Ok(self.read_byte()? as i8)
    }

    /// Alias for `get_i8`
    pub fn get_byte(&mut self) -> Result<i8, Error> {
        self.get_i8()
    }

    pub fn get_u8(&mut self) -> Result<u8, Error> {
        self.read_byte()
    }

    pub fn get_i16(&mut self) -> Result<i16, Error> {
        Ok(self.read_zigzag()? as i16)
    }

    /// Alias for `get_i16`
    pub fn get_short(&mut self) -> Result<i16, Error> {
        self.get_i16()
    }

    pub fn get_u16(&mut self) -> Result<u16, Error> {
        Ok(self.read_varint()? as u16)
    }

    pub fn get_i32(&mut self) -> Result<i32, Error> {
        Ok(self.read_zigzag()? as i32)
    }

    /// Alias for `get_i32`
    pub fn get_int(&mut self) -> Result<i32, Error> {
        self.get_i32()
    }

    pub fn get_u32(&mut self) -> Result<u32, Error> {
        Ok(self.read_varint()? as u32)
    }

    pub fn get_i64(&mut self) -> Result<i64, Error> {
        self.read_zigzag()
    }

    /// Alias for `get_i64`
    pub fn get_long(&mut self) -> Result<i64, Error> {
        self.get_i64()
    }

    pub fn get_u64(&mut self) -> Result<u64, Error> {
        self.read_varint()
    }

    pub fn get_f32(&mut self) -> Result<f32, Error> {
        if self.read_pos + 4 > self.data.len() {
            return Err(Error::SerdeError("EOF".to_string()));
        }
        let mut b = [0u8; 4];
        b.copy_from_slice(&self.data[self.read_pos..self.read_pos + 4]);
        self.read_pos += 4;
        Ok(f32::from_le_bytes(b))
    }

    /// Alias for `get_f32`
    pub fn get_float(&mut self) -> Result<f32, Error> {
        self.get_f32()
    }

    pub fn get_f64(&mut self) -> Result<f64, Error> {
        if self.read_pos + 8 > self.data.len() {
            return Err(Error::SerdeError("EOF".to_string()));
        }
        let mut b = [0u8; 8];
        b.copy_from_slice(&self.data[self.read_pos..self.read_pos + 8]);
        self.read_pos += 8;
        Ok(f64::from_le_bytes(b))
    }

    /// Alias for `get_f64`
    pub fn get_double(&mut self) -> Result<f64, Error> {
        self.get_f64()
    }

    pub fn get_string(&mut self) -> Result<String, Error> {
        let len = self.read_varint()? as usize;
        if self.read_pos + len > self.data.len() {
            return Err(Error::SerdeError("EOF".to_string()));
        }
        let s = std::str::from_utf8(&self.data[self.read_pos..self.read_pos + len])
            .map_err(|e| Error::SerdeError(e.to_string()))?;
        self.read_pos += len;
        Ok(s.to_string())
    }

    pub fn get_bytes(&mut self) -> Result<Vec<u8>, Error> {
        let len = self.read_varint()? as usize;
        if self.read_pos + len > self.data.len() {
            return Err(Error::SerdeError("EOF".to_string()));
        }
        let b = self.data[self.read_pos..self.read_pos + len].to_vec();
        self.read_pos += len;
        Ok(b)
    }

    pub fn get_uuid(&mut self) -> Result<uuid::Uuid, Error> {
        if self.read_pos + 16 > self.data.len() {
            return Err(Error::SerdeError("EOF".to_string()));
        }
        let mut b = [0u8; 16];
        b.copy_from_slice(&self.data[self.read_pos..self.read_pos + 16]);
        self.read_pos += 16;
        Ok(uuid::Uuid::from_bytes(b))
    }
}

pub struct Serializer {
    output: Vec<u8>,
}

impl Serializer {
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            output: Vec::with_capacity(1024),
        }
    }

    #[inline]
    fn write_varint(&mut self, mut value: u64) {
        loop {
            let byte = (value & 0x7f) as u8;
            value >>= 7;
            if value == 0 {
                self.output.push(byte);
                break;
            }
            self.output.push(byte | 0x80);
        }
    }

    #[inline]
    fn write_zigzag(&mut self, value: i64) {
        let encoded = ((value << 1) ^ (value >> 63)) as u64;
        self.write_varint(encoded);
    }
}

impl Default for Serializer {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl ser::Serializer for &mut Serializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<(), Error> {
        self.output.push(u8::from(v));
        Ok(())
    }
    #[inline]
    fn serialize_i8(self, v: i8) -> Result<(), Error> {
        self.output.push(v as u8);
        Ok(())
    }
    #[inline]
    fn serialize_i16(self, v: i16) -> Result<(), Error> {
        self.write_zigzag(i64::from(v));
        Ok(())
    }
    #[inline]
    fn serialize_i32(self, v: i32) -> Result<(), Error> {
        self.write_zigzag(i64::from(v));
        Ok(())
    }
    #[inline]
    fn serialize_i64(self, v: i64) -> Result<(), Error> {
        self.write_zigzag(v);
        Ok(())
    }
    fn serialize_u8(self, v: u8) -> Result<(), Error> {
        self.output.push(v);
        Ok(())
    }
    fn serialize_u16(self, v: u16) -> Result<(), Error> {
        self.write_varint(u64::from(v));
        Ok(())
    }
    fn serialize_u32(self, v: u32) -> Result<(), Error> {
        self.write_varint(u64::from(v));
        Ok(())
    }
    fn serialize_u64(self, v: u64) -> Result<(), Error> {
        self.write_varint(v);
        Ok(())
    }
    #[inline]
    fn serialize_f32(self, v: f32) -> Result<(), Error> {
        self.output.extend_from_slice(&v.to_le_bytes());
        Ok(())
    }
    #[inline]
    fn serialize_f64(self, v: f64) -> Result<(), Error> {
        self.output.extend_from_slice(&v.to_le_bytes());
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<(), Error> {
        self.serialize_str(&v.to_string())
    }
    #[inline]
    fn serialize_str(self, v: &str) -> Result<(), Error> {
        self.write_varint(v.len() as u64);
        self.output.extend_from_slice(v.as_bytes());
        Ok(())
    }
    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<(), Error> {
        self.write_varint(v.len() as u64);
        self.output.extend_from_slice(v);
        Ok(())
    }
    fn serialize_none(self) -> Result<(), Error> {
        self.output.push(0);
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), Error> {
        self.output.push(1);
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), Error> {
        Ok(())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Error> {
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        idx: u32,
        _variant: &'static str,
    ) -> Result<(), Error> {
        self.write_varint(u64::from(idx));
        Ok(())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        idx: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        if name == NBT_ARRAY_TAG {
            match variant {
                NBT_BYTE_ARRAY_TAG | NBT_INT_ARRAY_TAG | NBT_LONG_ARRAY_TAG => {
                    // Positional: skip indices/tags, just write data
                    return value.serialize(self);
                }
                _ => {}
            }
        }
        self.write_varint(u64::from(idx));
        value.serialize(self)
    }
    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        let len = len.ok_or_else(|| Error::SerdeError("Length required".to_string()))?;
        self.write_varint(len as u64);
        Ok(self)
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Error> {
        Ok(self)
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
        Ok(self)
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        Ok(self)
    }
    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        let len = len.ok_or_else(|| Error::SerdeError("Length required".to_string()))?;
        self.write_varint(len as u64);
        Ok(self)
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Error> {
        Ok(self)
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        Ok(self)
    }
}

impl ser::SerializeSeq for &mut Serializer {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}
impl ser::SerializeTuple for &mut Serializer {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}
impl ser::SerializeTupleStruct for &mut Serializer {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}
impl ser::SerializeTupleVariant for &mut Serializer {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}
impl ser::SerializeStruct for &mut Serializer {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}
impl ser::SerializeStructVariant for &mut Serializer {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}
impl ser::SerializeMap for &mut Serializer {
    type Ok = ();
    type Error = Error;
    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Error> {
        key.serialize(&mut **self)
    }
    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

pub struct Deserializer<'de> {
    input: &'de [u8],
}

impl<'de> Deserializer<'de> {
    #[must_use]
    pub const fn new(input: &'de [u8]) -> Self {
        Self { input }
    }
    fn read_byte(&mut self) -> Result<u8, Error> {
        if self.input.is_empty() {
            return Err(Error::SerdeError("EOF".to_string()));
        }
        let b = self.input[0];
        self.input = &self.input[1..];
        Ok(b)
    }
    fn read_varint(&mut self) -> Result<u64, Error> {
        let mut value = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_byte()?;
            value |= ((byte & 0x7f) as u64) << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }
        Ok(value)
    }
    fn read_zigzag(&mut self) -> Result<i64, Error> {
        let value = self.read_varint()?;
        Ok(((value >> 1) as i64) ^ (-((value & 1) as i64)))
    }
}

impl<'de> de::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = Error;
    fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Error> {
        Err(Error::SerdeError("Positional PNBT needs types".to_string()))
    }
    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_bool(self.read_byte()? != 0)
    }
    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_i8(self.read_byte()? as i8)
    }
    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_i16(self.read_zigzag()? as i16)
    }
    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_i32(self.read_zigzag()? as i32)
    }
    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_i64(self.read_zigzag()?)
    }
    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_u8(self.read_byte()?)
    }
    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_u16(self.read_varint()? as u16)
    }
    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_u32(self.read_varint()? as u32)
    }
    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_u64(self.read_varint()?)
    }
    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        if self.input.len() < 4 {
            return Err(Error::SerdeError("EOF".to_string()));
        }
        let mut b = [0u8; 4];
        b.copy_from_slice(&self.input[..4]);
        self.input = &self.input[4..];
        visitor.visit_f32(f32::from_le_bytes(b))
    }
    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        if self.input.len() < 8 {
            return Err(Error::SerdeError("EOF".to_string()));
        }
        let mut b = [0u8; 8];
        b.copy_from_slice(&self.input[..8]);
        self.input = &self.input[8..];
        visitor.visit_f64(f64::from_le_bytes(b))
    }
    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }
    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let len = self.read_varint()? as usize;
        if self.input.len() < len {
            return Err(Error::SerdeError("EOF".to_string()));
        }
        let s = std::str::from_utf8(&self.input[..len])
            .map_err(|e| Error::SerdeError(e.to_string()))?;
        self.input = &self.input[len..];
        visitor.visit_borrowed_str(s)
    }
    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }
    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let len = self.read_varint()? as usize;
        if self.input.len() < len {
            return Err(Error::SerdeError("EOF".to_string()));
        }
        let b = &self.input[..len];
        self.input = &self.input[len..];
        visitor.visit_borrowed_bytes(b)
    }
    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_bytes(visitor)
    }
    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        if self.read_byte()? == 0 {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }
    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_unit()
    }
    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        visitor.visit_unit()
    }
    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        visitor.visit_newtype_struct(self)
    }
    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let len = self.read_varint()? as usize;
        visitor.visit_seq(RawSeq { de: self, len })
    }
    fn deserialize_tuple<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_seq(RawSeq { de: self, len })
    }
    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Error> {
        visitor.visit_seq(RawSeq { de: self, len })
    }
    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let len = self.read_varint()? as usize;
        visitor.visit_map(RawMap { de: self, len })
    }
    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        visitor.visit_seq(RawSeq {
            de: self,
            len: fields.len(),
        })
    }
    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Error> {
        Err(Error::SerdeError("Unimplemented".to_string()))
    }
    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }
    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_unit()
    }
}

struct RawSeq<'a, 'de> {
    de: &'a mut Deserializer<'de>,
    len: usize,
}
impl<'de> SeqAccess<'de> for RawSeq<'_, 'de> {
    type Error = Error;
    fn next_element_seed<E: de::DeserializeSeed<'de>>(
        &mut self,
        seed: E,
    ) -> Result<Option<E::Value>, Error> {
        if self.len == 0 {
            return Ok(None);
        }
        self.len -= 1;
        seed.deserialize(&mut *self.de).map(Some)
    }
}

struct RawMap<'a, 'de> {
    de: &'a mut Deserializer<'de>,
    len: usize,
}
impl<'de> MapAccess<'de> for RawMap<'_, 'de> {
    type Error = Error;
    fn next_key_seed<K: de::DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Error> {
        if self.len == 0 {
            return Ok(None);
        }
        self.len -= 1;
        seed.deserialize(&mut *self.de).map(Some)
    }
    fn next_value_seed<V: de::DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Error> {
        seed.deserialize(&mut *self.de)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        a: i32,
        b: String,
        c: Vec<i8>,
        d: Inner,
        active: bool,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Inner {
        e: f32,
    }

    #[test]
    fn pnbt_serde() {
        let t = TestStruct {
            a: 123456,
            b: "hello world".to_string(),
            c: vec![1, 2, 3, 4, 5],
            d: Inner {
                e: std::f32::consts::PI,
            },
            active: true,
        };

        let bytes = to_pnbt(&t).unwrap();
        let decoded: TestStruct = from_pnbt(&bytes).unwrap();
        assert_eq!(t, decoded);
    }

    #[test]
    fn nbt_arrays_pnbt() {
        use crate::{nbt_byte_array, nbt_int_array, nbt_long_array};

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct ArrayStruct {
            #[serde(serialize_with = "nbt_byte_array")]
            b: Vec<u8>,
            #[serde(serialize_with = "nbt_int_array")]
            i: Vec<i32>,
            #[serde(serialize_with = "nbt_long_array")]
            l: Vec<i64>,
        }

        let t = ArrayStruct {
            b: vec![1, 2, 3],
            i: vec![100, 200, 300],
            l: vec![1000, 2000, 3000],
        };

        let bytes = to_pnbt(&t).unwrap();
        let decoded: ArrayStruct = from_pnbt(&bytes).unwrap();
        assert_eq!(t, decoded);
    }

    #[test]
    fn pnbt_compound_manual() {
        let mut compound = PNbtCompound::new();
        compound.put_i32(123456);
        compound.put_string("manual pnbt");
        compound.put_bool(true);
        compound.put_f32(std::f32::consts::PI);

        let bytes = compound.into_bytes();
        let mut reader = PNbtCompound::from_bytes(bytes);

        assert_eq!(reader.get_i32().unwrap(), 123456);
        assert_eq!(reader.get_string().unwrap(), "manual pnbt");
        assert!(reader.get_bool().unwrap());
        assert!((reader.get_f32().unwrap() - std::f32::consts::PI).abs() < 0.001);
    }
}
