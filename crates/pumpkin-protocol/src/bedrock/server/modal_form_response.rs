use crate::codec::var_int::VarInt;
use crate::serial::{PacketRead, PacketReadSlice, read_str_slice};
use pumpkin_macros::packet;
use std::borrow::Cow;
use std::io::{Error, Read};

#[derive(Debug)]
#[packet(101)]
pub struct SModalFormResponse<'a> {
    pub form_id: VarInt,
    pub form_data: Option<Cow<'a, str>>,
}

impl<'a> PacketReadSlice<'a> for SModalFormResponse<'a> {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let form_id = VarInt::read_slice(buf)?;
        let form_data = bool::read_slice(buf)?
            .then(|| read_str_slice(buf).map(Cow::Borrowed))
            .transpose()?;
        Ok(Self { form_id, form_data })
    }
}

impl PacketRead for SModalFormResponse<'static> {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let form_id = VarInt::read(reader)?;
        let form_data = Option::<String>::read(reader)?.map(Cow::Owned);
        Ok(Self { form_id, form_data })
    }
}
