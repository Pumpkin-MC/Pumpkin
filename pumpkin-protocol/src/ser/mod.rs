use core::str;
use std::io::{Read, Write};

use crate::{
    FixedBitSet,
    codec::{bit_set::BitSet, u24::U24, var_int::VarInt, var_long::VarLong, var_uint::VarUInt},
};

pub mod deserializer;
use pumpkin_nbt::{serializer::WriteAdaptor, tag::NbtTag};
use pumpkin_util::resource_location::ResourceLocation;
use thiserror::Error;
pub mod serializer;

// TODO: This is a bit hacky
const NO_PREFIX_MARKER: &str = "__network_no_prefix";

pub fn network_serialize_no_prefix<T, S>(input: T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: serde::Serialize,
    S: serde::Serializer,
{
    serializer.serialize_newtype_struct(NO_PREFIX_MARKER, &input)
}

#[derive(Debug, Error)]
pub enum ReadingError {
    #[error("EOF, Tried to read {0} but No bytes left to consume")]
    CleanEOF(String),
    #[error("incomplete: {0}")]
    Incomplete(String),
    #[error("too large: {0}")]
    TooLarge(String),
    #[error("{0}")]
    Message(String),
}

#[derive(Debug, Error)]
pub enum WritingError {
    #[error("IO error: {0}")]
    IoError(std::io::Error),
    #[error("Serde failure: {0}")]
    Serde(String),
    #[error("Failed to serialize packet: {0}")]
    Message(String),
}

pub trait NetworkReadExt {
    fn get_i8(&mut self) -> Result<i8, ReadingError>;
    fn get_u8(&mut self) -> Result<u8, ReadingError>;

    fn get_i16_be(&mut self) -> Result<i16, ReadingError>;
    fn get_u16_be(&mut self) -> Result<u16, ReadingError>;
    fn get_i32_be(&mut self) -> Result<i32, ReadingError>;
    fn get_u32_be(&mut self) -> Result<u32, ReadingError>;
    fn get_i64_be(&mut self) -> Result<i64, ReadingError>;
    fn get_u64_be(&mut self) -> Result<u64, ReadingError>;
    fn get_f32_be(&mut self) -> Result<f32, ReadingError>;
    fn get_f64_be(&mut self) -> Result<f64, ReadingError>;
    fn get_i128_be(&mut self) -> Result<i128, ReadingError>;
    fn get_u128_be(&mut self) -> Result<u128, ReadingError>;
    fn read_boxed_slice(&mut self, count: usize) -> Result<Box<[u8]>, ReadingError>;

    fn read_remaining_to_boxed_slice(&mut self, bound: usize) -> Result<Box<[u8]>, ReadingError>;

    fn get_bool(&mut self) -> Result<bool, ReadingError>;
    fn get_u24(&mut self) -> Result<U24, ReadingError>;
    fn get_var_int(&mut self) -> Result<VarInt, ReadingError>;
    fn get_var_uint(&mut self) -> Result<VarUInt, ReadingError>;
    fn get_var_long(&mut self) -> Result<VarLong, ReadingError>;
    fn get_string_bounded(&mut self, bound: usize) -> Result<String, ReadingError>;
    fn get_string(&mut self) -> Result<String, ReadingError>;
    fn get_resource_location(&mut self) -> Result<ResourceLocation, ReadingError>;
    fn get_uuid(&mut self) -> Result<uuid::Uuid, ReadingError>;
    fn get_fixed_bitset(&mut self, bits: usize) -> Result<FixedBitSet, ReadingError>;

    fn get_option<G>(
        &mut self,
        parse: impl FnOnce(&mut Self) -> Result<G, ReadingError>,
    ) -> Result<Option<G>, ReadingError>;

    fn get_list<G>(
        &mut self,
        parse: impl Fn(&mut Self) -> Result<G, ReadingError>,
    ) -> Result<Vec<G>, ReadingError>;
}

macro_rules! get_number_be {
    ($name:ident, $type:ty) => {
        fn $name(&mut self) -> Result<$type, ReadingError> {
            let mut buf = [0u8; std::mem::size_of::<$type>()];
            self.read_exact(&mut buf)
                .map_err(|err| ReadingError::Incomplete(err.to_string()))?;
            Ok(<$type>::from_be_bytes(buf))
        }
    };
}

impl<R: Read> NetworkReadExt for R {
    //TODO: Macroize this
    fn get_i8(&mut self) -> Result<i8, ReadingError> {
        let mut buf = [0u8];
        self.read_exact(&mut buf)
            .map_err(|err| ReadingError::Incomplete(err.to_string()))?;

        Ok(buf[0] as i8)
    }

    fn get_u8(&mut self) -> Result<u8, ReadingError> {
        let mut buf = [0u8];
        self.read_exact(&mut buf)
            .map_err(|err| ReadingError::Incomplete(err.to_string()))?;

        Ok(buf[0])
    }

    fn get_u24(&mut self) -> Result<U24, ReadingError> {
        U24::decode(self)
    }

    get_number_be!(get_i16_be, i16);
    get_number_be!(get_u16_be, u16);
    get_number_be!(get_i32_be, i32);
    get_number_be!(get_u32_be, u32);
    get_number_be!(get_i64_be, i64);
    get_number_be!(get_u64_be, u64);
    get_number_be!(get_i128_be, i128);
    get_number_be!(get_u128_be, u128);
    get_number_be!(get_f32_be, f32);
    get_number_be!(get_f64_be, f64);

    fn read_boxed_slice(&mut self, count: usize) -> Result<Box<[u8]>, ReadingError> {
        let mut buf = vec![0u8; count];
        self.read_exact(&mut buf)
            .map_err(|err| ReadingError::Incomplete(err.to_string()))?;

        Ok(buf.into())
    }

    fn read_remaining_to_boxed_slice(&mut self, bound: usize) -> Result<Box<[u8]>, ReadingError> {
        let mut return_buf = Vec::new();

        // TODO: We can probably remove the temp buffer somehow
        let mut temp_buf = [0; 1024];
        loop {
            let bytes_read = self
                .read(&mut temp_buf)
                .map_err(|err| ReadingError::Incomplete(err.to_string()))?;

            if bytes_read == 0 {
                break;
            }

            if return_buf.len() + bytes_read > bound {
                return Err(ReadingError::TooLarge(
                    "Read remaining too long".to_string(),
                ));
            }

            return_buf.extend(&temp_buf[..bytes_read]);
        }
        Ok(return_buf.into_boxed_slice())
    }

    fn get_bool(&mut self) -> Result<bool, ReadingError> {
        let byte = self.get_u8()?;
        Ok(byte != 0)
    }

    fn get_var_int(&mut self) -> Result<VarInt, ReadingError> {
        VarInt::decode(self)
    }
    fn get_var_uint(&mut self) -> Result<VarUInt, ReadingError> {
        VarUInt::decode(self)
    }

    fn get_var_long(&mut self) -> Result<VarLong, ReadingError> {
        VarLong::decode(self)
    }

    fn get_string_bounded(&mut self, bound: usize) -> Result<String, ReadingError> {
        let size = self.get_var_uint()?.0 as usize;
        if size > bound {
            return Err(ReadingError::TooLarge("string".to_string()));
        }

        let data = self.read_boxed_slice(size)?;
        String::from_utf8(data.into()).map_err(|e| ReadingError::Message(e.to_string()))
    }

    fn get_string(&mut self) -> Result<String, ReadingError> {
        self.get_string_bounded(i32::MAX as usize)
    }

    fn get_resource_location(&mut self) -> Result<ResourceLocation, ReadingError> {
        let resource_location = self.get_string_bounded(ResourceLocation::MAX_SIZE.get())?;
        match resource_location.split_once(":") {
            Some((namespace, path)) => Ok(ResourceLocation {
                namespace: namespace.to_string(),
                path: path.to_string(),
            }),
            None => Err(ReadingError::Incomplete("ResourceLocation".to_string())),
        }
    }

    fn get_uuid(&mut self) -> Result<uuid::Uuid, ReadingError> {
        let mut bytes = [0u8; 16];
        self.read_exact(&mut bytes)
            .map_err(|err| ReadingError::Incomplete(err.to_string()))?;
        Ok(uuid::Uuid::from_slice(&bytes).expect("Failed to parse UUID"))
    }

    fn get_fixed_bitset(&mut self, bits: usize) -> Result<FixedBitSet, ReadingError> {
        let bytes = self.read_boxed_slice(bits.div_ceil(8))?;
        Ok(bytes)
    }

    fn get_option<G>(
        &mut self,
        parse: impl FnOnce(&mut Self) -> Result<G, ReadingError>,
    ) -> Result<Option<G>, ReadingError> {
        if self.get_bool()? {
            Ok(Some(parse(self)?))
        } else {
            Ok(None)
        }
    }

    fn get_list<G>(
        &mut self,
        parse: impl Fn(&mut Self) -> Result<G, ReadingError>,
    ) -> Result<Vec<G>, ReadingError> {
        let len = self.get_var_int()?.0 as usize;
        let mut list = Vec::with_capacity(len);
        for _ in 0..len {
            list.push(parse(self)?);
        }
        Ok(list)
    }
}

pub trait NetworkWriteExt {
    fn write_i8(&mut self, data: i8) -> Result<(), WritingError>;
    fn write_u8(&mut self, data: u8) -> Result<(), WritingError>;
    fn write_i16_be(&mut self, data: i16) -> Result<(), WritingError>;
    fn write_u16_be(&mut self, data: u16) -> Result<(), WritingError>;
    fn write_u24_be(&mut self, data: U24) -> Result<(), WritingError>;
    fn write_i32_be(&mut self, data: i32) -> Result<(), WritingError>;
    fn write_u32_be(&mut self, data: u32) -> Result<(), WritingError>;
    fn write_i64_be(&mut self, data: i64) -> Result<(), WritingError>;
    fn write_u64_be(&mut self, data: u64) -> Result<(), WritingError>;
    fn write_f32_be(&mut self, data: f32) -> Result<(), WritingError>;
    fn write_f64_be(&mut self, data: f64) -> Result<(), WritingError>;
    fn write_slice(&mut self, data: &[u8]) -> Result<(), WritingError>;

    fn write_bool(&mut self, data: bool) -> Result<(), WritingError> {
        if data {
            self.write_u8(1)
        } else {
            self.write_u8(0)
        }
    }
    fn write_var_int(&mut self, data: &VarInt) -> Result<(), WritingError>;
    fn write_var_uint(&mut self, data: &VarUInt) -> Result<(), WritingError>;
    fn write_var_long(&mut self, data: &VarLong) -> Result<(), WritingError>;
    fn write_string_bounded(&mut self, data: &str, bound: usize) -> Result<(), WritingError>;
    fn write_string(&mut self, data: &str) -> Result<(), WritingError>;
    fn write_resource_location(&mut self, data: &ResourceLocation) -> Result<(), WritingError>;

    fn write_uuid(&mut self, data: &uuid::Uuid) -> Result<(), WritingError> {
        let (first, second) = data.as_u64_pair();
        self.write_u64_be(first)?;
        self.write_u64_be(second)
    }

    fn write_bitset(&mut self, bitset: &BitSet) -> Result<(), WritingError>;

    fn write_option<G>(
        &mut self,
        data: &Option<G>,
        writer: impl FnOnce(&mut Self, &G) -> Result<(), WritingError>,
    ) -> Result<(), WritingError> {
        if let Some(data) = data {
            self.write_bool(true)?;
            writer(self, data)
        } else {
            self.write_bool(false)
        }
    }

    fn write_list<G>(
        &mut self,
        list: &[G],
        writer: impl Fn(&mut Self, &G) -> Result<(), WritingError>,
    ) -> Result<(), WritingError> {
        self.write_var_int(&list.len().try_into().map_err(|_| {
            WritingError::Message(format!("{} isn't representable as a VarInt", list.len()))
        })?)?;

        for data in list {
            writer(self, data)?;
        }

        Ok(())
    }

    fn write_nbt(&mut self, data: &NbtTag) -> Result<(), WritingError>;
}

macro_rules! write_number_be {
    ($name:ident, $type:ty) => {
        fn $name(&mut self, data: $type) -> Result<(), WritingError> {
            self.write_all(&data.to_be_bytes())
                .map_err(WritingError::IoError)
        }
    };
}

impl<W: Write> NetworkWriteExt for W {
    fn write_i8(&mut self, data: i8) -> Result<(), WritingError> {
        self.write_all(&data.to_be_bytes())
            .map_err(WritingError::IoError)
    }

    fn write_u8(&mut self, data: u8) -> Result<(), WritingError> {
        self.write_all(&data.to_be_bytes())
            .map_err(WritingError::IoError)
    }

    fn write_u24_be(&mut self, data: U24) -> Result<(), WritingError> {
        data.encode(self)
    }

    write_number_be!(write_i16_be, i16);
    write_number_be!(write_u16_be, u16);
    write_number_be!(write_i32_be, i32);
    write_number_be!(write_u32_be, u32);
    write_number_be!(write_i64_be, i64);
    write_number_be!(write_u64_be, u64);
    write_number_be!(write_f32_be, f32);
    write_number_be!(write_f64_be, f64);

    fn write_slice(&mut self, data: &[u8]) -> Result<(), WritingError> {
        self.write_all(data).map_err(WritingError::IoError)
    }

    fn write_var_int(&mut self, data: &VarInt) -> Result<(), WritingError> {
        data.encode(self)
    }

    fn write_var_uint(&mut self, data: &VarUInt) -> Result<(), WritingError> {
        data.encode(self)
    }

    fn write_var_long(&mut self, data: &VarLong) -> Result<(), WritingError> {
        data.encode(self)
    }

    fn write_string_bounded(&mut self, data: &str, bound: usize) -> Result<(), WritingError> {
        assert!(data.len() <= bound);
        self.write_var_int(&data.len().try_into().map_err(|_| {
            WritingError::Message(format!("{} isn't representable as a VarInt", data.len()))
        })?)?;

        self.write_all(data.as_bytes())
            .map_err(WritingError::IoError)
    }

    fn write_string(&mut self, data: &str) -> Result<(), WritingError> {
        self.write_string_bounded(data, i16::MAX as usize)
    }

    fn write_resource_location(&mut self, data: &ResourceLocation) -> Result<(), WritingError> {
        self.write_string_bounded(&data.to_string(), ResourceLocation::MAX_SIZE.get())
    }

    fn write_bitset(&mut self, data: &BitSet) -> Result<(), WritingError> {
        data.encode(self)
    }

    fn write_option<G>(
        &mut self,
        data: &Option<G>,
        writer: impl FnOnce(&mut Self, &G) -> Result<(), WritingError>,
    ) -> Result<(), WritingError> {
        if let Some(data) = data {
            self.write_bool(true)?;
            writer(self, data)
        } else {
            self.write_bool(false)
        }
    }

    fn write_list<G>(
        &mut self,
        list: &[G],
        writer: impl Fn(&mut Self, &G) -> Result<(), WritingError>,
    ) -> Result<(), WritingError> {
        self.write_var_int(&(list.len() as i32).into())?;
        for data in list {
            writer(self, data)?;
        }

        Ok(())
    }

    fn write_nbt(&mut self, data: &NbtTag) -> Result<(), WritingError> {
        let mut write_adaptor = WriteAdaptor::new(self);
        data.serialize(&mut write_adaptor)
            .map_err(|e| WritingError::Message(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use serde::{Deserialize, Serialize};

    use crate::{
        VarInt,
        ser::{deserializer, serializer},
    };

    #[test]
    fn test_i32_reserialize() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
        struct Foo {
            bar: i32,
        }
        let foo = Foo { bar: 69 };
        let mut bytes = Vec::new();
        let mut serializer = serializer::Serializer::new(&mut bytes);
        foo.serialize(&mut serializer).unwrap();

        let cursor = Cursor::new(bytes);
        let deserialized: Foo =
            Foo::deserialize(&mut deserializer::Deserializer::new(cursor)).unwrap();

        assert_eq!(foo, deserialized);
    }

    #[test]
    fn test_varint_reserialize() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
        struct Foo {
            bar: VarInt,
        }
        let foo = Foo { bar: 69.into() };
        let mut bytes = Vec::new();
        let mut serializer = serializer::Serializer::new(&mut bytes);
        foo.serialize(&mut serializer).unwrap();

        let cursor = Cursor::new(bytes);
        let deserialized: Foo =
            Foo::deserialize(&mut deserializer::Deserializer::new(cursor)).unwrap();

        assert_eq!(foo, deserialized);
    }

    #[test]
    fn test_char_reserialize() {
        #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
        struct CharStruct {
            c: char,
        }

        // Test with normal char
        let original = CharStruct { c: 'A' };
        let mut bytes = Vec::new();
        let mut ser = serializer::Serializer::new(&mut bytes);
        original.serialize(&mut ser).unwrap();
        assert_eq!(bytes, vec![0, 0, 0, 0x41]);

        let de_cursor = Cursor::new(bytes);
        let deserialized: CharStruct =
            CharStruct::deserialize(&mut deserializer::Deserializer::new(de_cursor)).unwrap();
        assert_eq!(original, deserialized);

        // Test with complex char
        let original_complex = CharStruct { c: 'Ω' }; // Greek Omega, U+03A9
        let mut bytes_complex = Vec::new();
        let mut ser_complex = serializer::Serializer::new(&mut bytes_complex);
        original_complex.serialize(&mut ser_complex).unwrap();
        assert_eq!(bytes_complex, vec![0, 0, 0x03, 0xA9]);

        let de_cursor_complex = Cursor::new(bytes_complex);
        let deserialized_complex: CharStruct =
            CharStruct::deserialize(&mut deserializer::Deserializer::new(de_cursor_complex))
                .unwrap();
        assert_eq!(original_complex, deserialized_complex);

        // Test with an emoji
        let original_emoji = CharStruct { c: '\u{1F383}' }; // Pumpkin emoji, U+1F383
        let mut bytes_emoji = Vec::new();
        let mut ser_emoji = serializer::Serializer::new(&mut bytes_emoji);
        original_emoji.serialize(&mut ser_emoji).unwrap();
        assert_eq!(bytes_emoji, vec![0, 0x01, 0xF3, 0x83]);

        let de_cursor_emoji = Cursor::new(bytes_emoji);
        let deserialized_emoji: CharStruct =
            CharStruct::deserialize(&mut deserializer::Deserializer::new(de_cursor_emoji)).unwrap();
        assert_eq!(original_emoji, deserialized_emoji);
    }

    #[test]
    fn test_i128_reserialize() {
        #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
        struct I128Struct {
            val: i128,
        }

        let original = I128Struct {
            val: 12345678901234567890123456789012345678,
        };
        let mut bytes = Vec::new();
        let mut ser = serializer::Serializer::new(&mut bytes);
        original.serialize(&mut ser).unwrap();
        assert_eq!(bytes, original.val.to_be_bytes());

        let de_cursor = Cursor::new(bytes);
        let deserialized: I128Struct =
            I128Struct::deserialize(&mut deserializer::Deserializer::new(de_cursor)).unwrap();
        assert_eq!(original, deserialized);

        let original_neg = I128Struct {
            val: -12345678901234567890123456789012345678,
        };
        let mut bytes_neg = Vec::new();
        let mut ser_neg = serializer::Serializer::new(&mut bytes_neg);
        original_neg.serialize(&mut ser_neg).unwrap();
        assert_eq!(bytes_neg, original_neg.val.to_be_bytes());

        let de_cursor_neg = Cursor::new(bytes_neg);
        let deserialized_neg: I128Struct =
            I128Struct::deserialize(&mut deserializer::Deserializer::new(de_cursor_neg)).unwrap();
        assert_eq!(original_neg, deserialized_neg);
    }

    #[test]
    fn test_u128_reserialize() {
        #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
        struct U128Struct {
            val: u128,
        }

        let original = U128Struct {
            val: 123456789012345678901234567890123456789,
        };
        let mut bytes = Vec::new();
        let mut ser = serializer::Serializer::new(&mut bytes);
        original.serialize(&mut ser).unwrap();
        assert_eq!(bytes, original.val.to_be_bytes());

        let de_cursor = Cursor::new(bytes);
        let deserialized: U128Struct =
            U128Struct::deserialize(&mut deserializer::Deserializer::new(de_cursor)).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_unit_reserialize() {
        #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
        struct UnitStruct;

        let original = UnitStruct;
        let mut bytes = Vec::new();
        let mut ser = serializer::Serializer::new(&mut bytes);
        original.serialize(&mut ser).unwrap();
        assert!(bytes.is_empty());

        let de_cursor = Cursor::new(bytes);
        let deserialized: UnitStruct =
            UnitStruct::deserialize(&mut deserializer::Deserializer::new(de_cursor)).unwrap();
        assert_eq!(original, deserialized);

        #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
        struct StructWithUnit {
            a: i32,
            b: UnitStruct,
            c: i32,
        }

        let original_with_unit = StructWithUnit {
            a: 1,
            b: UnitStruct,
            c: 2,
        };
        let mut bytes_with_unit = Vec::new();
        let mut ser_with_unit = serializer::Serializer::new(&mut bytes_with_unit);
        original_with_unit.serialize(&mut ser_with_unit).unwrap();

        // Check that only a and c were serialized
        let mut expected_bytes = Vec::new();
        expected_bytes.extend_from_slice(&1i32.to_be_bytes());
        expected_bytes.extend_from_slice(&2i32.to_be_bytes());
        assert_eq!(bytes_with_unit, expected_bytes);

        let de_cursor_with_unit = Cursor::new(bytes_with_unit);
        let deserialized_with_unit: StructWithUnit =
            StructWithUnit::deserialize(&mut deserializer::Deserializer::new(de_cursor_with_unit))
                .unwrap();
        assert_eq!(original_with_unit, deserialized_with_unit);
    }

    #[test]
    fn test_enum_reserialize() {
        #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
        enum MyEnum {
            A,
            B(i32),
            C { x: i32, y: String },
        }

        let original_a = MyEnum::A;
        let mut bytes_a = Vec::new();
        let mut ser_a = serializer::Serializer::new(&mut bytes_a);
        original_a.serialize(&mut ser_a).unwrap();
        // VarInt for index 0
        assert_eq!(bytes_a, vec![0x00]);
        let de_cursor_a = Cursor::new(bytes_a);
        let deserialized_a: MyEnum =
            MyEnum::deserialize(&mut deserializer::Deserializer::new(de_cursor_a)).unwrap();
        assert_eq!(original_a, deserialized_a);

        let original_b = MyEnum::B(123);
        let mut bytes_b = Vec::new();
        let mut ser_b = serializer::Serializer::new(&mut bytes_b);
        original_b.serialize(&mut ser_b).unwrap();
        // VarInt for index 1, then i32 for 123
        let mut expected_bytes_b = vec![0x01];
        expected_bytes_b.extend_from_slice(&123i32.to_be_bytes());
        assert_eq!(bytes_b, expected_bytes_b);
        let de_cursor_b = Cursor::new(bytes_b);
        let deserialized_b: MyEnum =
            MyEnum::deserialize(&mut deserializer::Deserializer::new(de_cursor_b)).unwrap();
        assert_eq!(original_b, deserialized_b);

        let original_c = MyEnum::C {
            x: 456,
            y: "hello".to_string(),
        };
        let mut bytes_c = Vec::new();
        let mut ser_c = serializer::Serializer::new(&mut bytes_c);
        original_c.serialize(&mut ser_c).unwrap();
        // VarInt for index 2, then i32 for 456, then string "hello"
        let mut expected_bytes_c = vec![0x02];
        expected_bytes_c.extend_from_slice(&456i32.to_be_bytes());
        expected_bytes_c.push(0x05); // VarInt for string length 5
        expected_bytes_c.extend_from_slice("hello".as_bytes());
        assert_eq!(bytes_c, expected_bytes_c);
        let de_cursor_c = Cursor::new(bytes_c);
        let deserialized_c: MyEnum =
            MyEnum::deserialize(&mut deserializer::Deserializer::new(de_cursor_c)).unwrap();
        assert_eq!(original_c, deserialized_c);
    }

    #[test]
    fn test_tuple_struct_reserialize() {
        #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
        struct MyTupleStruct(i32, String);

        let original = MyTupleStruct(789, "world".to_string());
        let mut bytes = Vec::new();
        let mut ser = serializer::Serializer::new(&mut bytes);
        original.serialize(&mut ser).unwrap();

        let mut expected_bytes = Vec::new();
        expected_bytes.extend_from_slice(&789i32.to_be_bytes());
        expected_bytes.push(0x05); // VarInt for string length 5
        expected_bytes.extend_from_slice("world".as_bytes());
        assert_eq!(bytes, expected_bytes);

        let de_cursor = Cursor::new(bytes);
        let deserialized: MyTupleStruct =
            MyTupleStruct::deserialize(&mut deserializer::Deserializer::new(de_cursor)).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_map_reserialize() {
        use std::collections::HashMap;

        #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
        struct MyMapStruct {
            map: HashMap<String, i32>,
        }

        let mut map = HashMap::new();
        map.insert("one".to_string(), 1);
        map.insert("two".to_string(), 2);

        let original = MyMapStruct { map };

        let mut bytes = Vec::new();
        let mut serializer = serializer::Serializer::new(&mut bytes);
        original.serialize(&mut serializer).unwrap();

        // Expected bytes: VarInt for map length (2), then key1, value1, key2, value2
        // Order of elements in HashMap is not guaranteed, so we check deserialized content

        let de_cursor = Cursor::new(bytes.clone()); // Clone bytes for potential debug
        let deserialized: MyMapStruct =
            MyMapStruct::deserialize(&mut deserializer::Deserializer::new(de_cursor)).unwrap();

        assert_eq!(original.map.len(), deserialized.map.len());
        for (k, v) in original.map {
            assert_eq!(deserialized.map.get(&k), Some(&v));
        }

        // Test with an empty map
        let empty_map_original = MyMapStruct {
            map: HashMap::new(),
        };
        let mut empty_map_bytes = Vec::new();
        let mut empty_map_ser = serializer::Serializer::new(&mut empty_map_bytes);
        empty_map_original.serialize(&mut empty_map_ser).unwrap();
        assert_eq!(empty_map_bytes, vec![0x00]); // VarInt for length 0

        let empty_map_de_cursor = Cursor::new(empty_map_bytes);
        let empty_map_deserialized: MyMapStruct =
            MyMapStruct::deserialize(&mut deserializer::Deserializer::new(empty_map_de_cursor))
                .unwrap();
        assert_eq!(empty_map_original, empty_map_deserialized);
    }
}
