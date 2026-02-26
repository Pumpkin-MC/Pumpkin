use std::fmt::Display;

use crate::ser::NetworkReadExt;
use serde::de::{EnumAccess, IntoDeserializer, VariantAccess, Visitor};

use super::{Read, ReadingError};
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess};

pub struct Deserializer<R: Read> {
    inner: R,
}

impl de::Error for ReadingError {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}

impl<R: Read> Deserializer<R> {
    pub const fn new(read: R) -> Self {
        Self { inner: read }
    }
}

impl<'de, R: Read> de::Deserializer<'de> for &mut Deserializer<R> {
    type Error = ReadingError;

    fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        unimplemented!(
            "This is impossible to do, since you cannot infer the data structure from the packet"
        )
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_bool(self.inner.get_bool()?)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i8(self.inner.get_i8()?)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i16(self.inner.get_i16_be()?)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i32(self.inner.get_i32_be()?)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i64(self.inner.get_i64_be()?)
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u8(self.inner.get_u8()?)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u16(self.inner.get_u16_be()?)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u32(self.inner.get_u32_be()?)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u64(self.inner.get_u64_be()?)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_f32(self.inner.get_f32_be()?)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_f64(self.inner.get_f64_be()?)
    }

    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i128(self.inner.get_i128_be()?)
    }

    fn deserialize_u128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u128(self.inner.get_u128_be()?)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let u32_val = self.inner.get_u32_be()?;
        char::from_u32(u32_val).map_or_else(
            || {
                Err(ReadingError::Message(format!(
                    "Invalid char value: {u32_val}"
                )))
            },
            |c| visitor.visit_char(c),
        )
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_str(&self.inner.get_string()?)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let len = self.inner.get_var_int()?.0 as usize;
        let mut buf = vec![0u8; len];
        self.inner
            .read_exact(&mut buf)
            .map_err(|err| ReadingError::Message(err.to_string()))?;
        visitor.visit_byte_buf(buf)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        if self.inner.get_bool()? {
            visitor.visit_some(self)
        } else {
            visitor.visit_none()
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        unimplemented!()
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        struct Access<'a, R: Read> {
            deserializer: &'a mut Deserializer<R>,
        }

        impl<'de, R: Read> SeqAccess<'de> for Access<'_, R> {
            type Error = ReadingError;

            fn next_element_seed<T: DeserializeSeed<'de>>(
                &mut self,
                seed: T,
            ) -> Result<Option<T::Value>, Self::Error> {
                let value = DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
                Ok(Some(value))
            }
        }

        visitor.visit_seq(Access { deserializer: self })
    }

    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        struct Access<'a, R: Read> {
            deserializer: &'a mut Deserializer<R>,
            len: usize,
        }

        impl<'de, R: Read> SeqAccess<'de> for Access<'_, R> {
            type Error = ReadingError;

            fn next_element_seed<T: DeserializeSeed<'de>>(
                &mut self,
                seed: T,
            ) -> Result<Option<T::Value>, Self::Error> {
                if self.len > 0 {
                    self.len -= 1;
                    let value = DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }
        }

        visitor.visit_seq(Access {
            deserializer: self,
            len,
        })
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        struct Access<'a, R: Read> {
            deserializer: &'a mut Deserializer<R>,
            len: usize,
        }
        impl<'de, R: Read> MapAccess<'de> for Access<'_, R> {
            type Error = ReadingError;

            fn next_key_seed<K: DeserializeSeed<'de>>(
                &mut self,
                seed: K,
            ) -> Result<Option<K::Value>, Self::Error> {
                if self.len == 0 {
                    return Ok(None);
                }
                seed.deserialize(&mut *self.deserializer).map(Some)
            }

            fn next_value_seed<Val: DeserializeSeed<'de>>(
                &mut self,
                seed: Val,
            ) -> Result<Val::Value, Self::Error> {
                self.len -= 1;
                seed.deserialize(&mut *self.deserializer)
            }
        }
        let len = self.inner.get_var_int()?.0 as usize;

        visitor.visit_map(Access {
            deserializer: self,
            len,
        })
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_enum(self)
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        unimplemented!()
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(
        self,
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        unimplemented!()
    }
}

impl<'de, R: Read> EnumAccess<'de> for &mut Deserializer<R> {
    type Error = ReadingError;
    type Variant = Self;

    fn variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Self::Error> {
        let variant_index = self.inner.get_var_int()?.0;
        let variant_index_u32: u32 = variant_index.try_into().map_err(|_| {
            ReadingError::Message(format!(
                "Invalid variant index {variant_index} for enum, cannot convert to u32"
            ))
        })?;
        let val = seed.deserialize(variant_index_u32.into_deserializer())?;
        Ok((val, self))
    }
}

impl<'de, R: Read> VariantAccess<'de> for &mut Deserializer<R> {
    type Error = ReadingError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(
        self,
        seed: T,
    ) -> Result<T::Value, Self::Error> {
        seed.deserialize(self)
    }

    fn tuple_variant<V: Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        de::Deserializer::deserialize_tuple(self, len, visitor)
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        de::Deserializer::deserialize_struct(self, "", fields, visitor)
    }
}
