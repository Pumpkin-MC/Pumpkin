use crate::codec::var_int::VarInt;
use bytes::BufMut;
use pumpkin_data::Enchantment;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{
    DataComponentImpl, EnchantmentsImpl, MaxStackSizeImpl, get,
};
use serde::de;
use serde::de::SeqAccess;
use serde::ser::SerializeStruct;
use std::borrow::Cow;

trait DataComponentCodec<Impl: DataComponentImpl> {
    fn serialize<T: SerializeStruct>(&self, seq: &mut T) -> Result<(), T::Error>;
    fn hash_serialize(&self, seq: &mut Vec<u8>);
    fn deserialize<'a, A: SeqAccess<'a>>(seq: &mut A) -> Result<Impl, A::Error>;
}

impl DataComponentCodec<Self> for MaxStackSizeImpl {
    fn serialize<T: SerializeStruct>(&self, seq: &mut T) -> Result<(), T::Error> {
        seq.serialize_field::<VarInt>("", &VarInt::from(self.size))
    }
    fn hash_serialize(&self, seq: &mut Vec<u8>) {
        // net.minecraft.util.HashOps#createInt
        seq.push(8u8);
        seq.put_slice(&(self.size as i32).to_le_bytes());
    }
    fn deserialize<'a, A: SeqAccess<'a>>(seq: &mut A) -> Result<Self, A::Error> {
        let size = u8::try_from(
            seq.next_element::<VarInt>()?
                .ok_or(de::Error::custom("No MaxStackSize VarInt!"))?
                .0,
        )
        .map_err(|_| de::Error::custom("No MaxStackSize VarInt!"))?;
        Ok(Self { size })
    }
}

impl DataComponentCodec<Self> for EnchantmentsImpl {
    fn serialize<T: SerializeStruct>(&self, seq: &mut T) -> Result<(), T::Error> {
        seq.serialize_field::<VarInt>("", &VarInt::from(self.enchantment.len() as i32))?;
        for (enc, level) in self.enchantment.iter() {
            seq.serialize_field::<VarInt>("", &VarInt::from(enc.id))?;
            seq.serialize_field::<VarInt>("", &VarInt::from(*level))?;
        }
        Ok(())
    }
    fn hash_serialize(&self, _seq: &mut Vec<u8>) {
        todo!();
    }
    fn deserialize<'a, A: SeqAccess<'a>>(seq: &mut A) -> Result<Self, A::Error> {
        let len = seq
            .next_element::<VarInt>()?
            .ok_or(de::Error::custom("No EnchantmentsImpl len VarInt!"))?
            .0 as usize;
        let mut enc = Vec::with_capacity(len);
        for _ in 0..len {
            let id = seq
                .next_element::<VarInt>()?
                .ok_or(de::Error::custom("No EnchantmentsImpl id VarInt!"))?
                .0 as u8;
            let level = seq
                .next_element::<VarInt>()?
                .ok_or(de::Error::custom("No EnchantmentsImpl level VarInt!"))?
                .0;
            enc.push((
                Enchantment::from_id(id).ok_or(de::Error::custom(
                    "EnchantmentsImpl Enchantment VarInt Incorrect!",
                ))?,
                level,
            ))
        }
        Ok(Self {
            enchantment: Cow::from(enc),
        })
    }
}

pub fn deserialize<'a, A: SeqAccess<'a>>(
    id: DataComponent,
    seq: &mut A,
) -> Result<Box<dyn DataComponentImpl>, A::Error> {
    match id {
        DataComponent::MaxStackSize => Ok(MaxStackSizeImpl::deserialize(seq)?.to_dyn()),
        DataComponent::Enchantments => Ok(EnchantmentsImpl::deserialize(seq)?.to_dyn()),
        _ => todo!(),
    }
}
pub fn serialize<T: SerializeStruct>(
    id: DataComponent,
    value: &dyn DataComponentImpl,
    seq: &mut T,
) -> Result<(), T::Error> {
    match id {
        DataComponent::MaxStackSize => get::<MaxStackSizeImpl>(value).serialize(seq),
        DataComponent::Enchantments => get::<EnchantmentsImpl>(value).serialize(seq),
        _ => todo!(),
    }
}
pub fn hash_serialize(id: DataComponent, value: &dyn DataComponentImpl, seq: &mut Vec<u8>) {
    match id {
        DataComponent::MaxStackSize => get::<MaxStackSizeImpl>(value).hash_serialize(seq),
        DataComponent::Enchantments => get::<EnchantmentsImpl>(value).hash_serialize(seq),
        _ => todo!(),
    }
}
