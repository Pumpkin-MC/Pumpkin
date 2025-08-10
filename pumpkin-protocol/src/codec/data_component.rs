use crate::codec::var_int::VarInt;
use bytes::BufMut;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{DataComponentImpl, MaxStackSizeImpl, get};
use serde::de;
use serde::de::SeqAccess;
use serde::ser::SerializeStruct;

trait DataComponentCodec<Impl: DataComponentImpl> {
    fn serialize<T: SerializeStruct>(&self, seq: &mut T) -> Result<(), T::Error>;
    fn hash_serialize(&self, seq: &mut Vec<u8>);
    fn deserialize<'a, A: SeqAccess<'a>>(seq: &mut A) -> Result<Impl, A::Error>;
}

impl DataComponentCodec<MaxStackSizeImpl> for MaxStackSizeImpl {
    fn serialize<T: SerializeStruct>(&self, seq: &mut T) -> Result<(), T::Error> {
        seq.serialize_field::<VarInt>("", &VarInt::from(self.size))
    }
    fn hash_serialize(&self, seq: &mut Vec<u8>) {
        // net.minecraft.util.HashOps#createInt
        seq.push(8u8);
        seq.put_slice(&(self.size as i32).to_le_bytes());
    }
    fn deserialize<'a, A: SeqAccess<'a>>(seq: &mut A) -> Result<MaxStackSizeImpl, A::Error> {
        let size = u8::try_from(
            seq.next_element::<VarInt>()?
                .ok_or(de::Error::custom("No MaxStackSize VarInt!"))?
                .0,
        )
        .map_err(|_| de::Error::custom("No MaxStackSize VarInt!"))?;
        Ok(Self { size })
    }
}

pub fn deserialize<'a, A: SeqAccess<'a>>(
    id: DataComponent,
    seq: &mut A,
) -> Result<Box<dyn DataComponentImpl>, A::Error> {
    match id {
        DataComponent::MaxStackSize => Ok(MaxStackSizeImpl::deserialize(seq)?.to_dyn()),
        _ => todo!(),
    }
}
pub fn serialize<T: SerializeStruct>(
    id: DataComponent,
    value: &dyn DataComponentImpl,
    seq: &mut T,
) -> Result<(), T::Error> {
    match id {
        DataComponent::MaxStackSize => Ok(get::<MaxStackSizeImpl>(value).serialize(seq)?),
        _ => todo!(),
    }
}
pub fn hash_serialize(id: DataComponent, value: &dyn DataComponentImpl, seq: &mut Vec<u8>) {
    match id {
        DataComponent::MaxStackSize => get::<MaxStackSizeImpl>(value).hash_serialize(seq),
        _ => todo!(),
    }
}
