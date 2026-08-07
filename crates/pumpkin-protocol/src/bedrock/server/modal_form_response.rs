use crate::codec::var_uint::VarUInt;
use crate::serial::{PacketRead, PacketReadSlice, read_str_slice};
use pumpkin_macros::packet;
use std::borrow::Cow;
use std::io::{Error, Read};

#[derive(Debug)]
#[packet(101)]
pub struct SModalFormResponse<'a> {
    pub form_id: VarUInt,
    pub form_data: Option<Cow<'a, str>>,
    pub cancel_reason: Option<u8>,
}

impl<'a> PacketReadSlice<'a> for SModalFormResponse<'a> {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let form_id = VarUInt::read_slice(buf)?;
        let form_data = bool::read_slice(buf)?
            .then(|| read_str_slice(buf).map(Cow::Borrowed))
            .transpose()?;
        let cancel_reason = Option::<u8>::read_slice(buf)?;
        Ok(Self {
            form_id,
            form_data,
            cancel_reason,
        })
    }
}

impl PacketRead for SModalFormResponse<'static> {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let form_id = VarUInt::read(reader)?;
        let form_data = Option::<String>::read(reader)?.map(Cow::Owned);
        let cancel_reason = Option::<u8>::read(reader)?;
        Ok(Self {
            form_id,
            form_data,
            cancel_reason,
        })
    }
}
