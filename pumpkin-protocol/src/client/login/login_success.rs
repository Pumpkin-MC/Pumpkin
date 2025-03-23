use std::io::Write;

use pumpkin_data::packet::clientbound::LOGIN_LOGIN_FINISHED;
use pumpkin_macros::packet;

use crate::{
    ClientPacket, Property,
    ser::{NetworkWriteExt, WritingError},
};

#[packet(LOGIN_LOGIN_FINISHED)]
pub struct CLoginSuccess<'a> {
    pub uuid: &'a uuid::Uuid,
    pub username: &'a str, // 16
    pub properties: &'a [Property],
}

impl<'a> CLoginSuccess<'a> {
    pub fn new(uuid: &'a uuid::Uuid, username: &'a str, properties: &'a [Property]) -> Self {
        Self {
            uuid,
            username,
            properties,
        }
    }
}

impl ClientPacket for CLoginSuccess<'_> {
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError> {
        let mut write = write;
        write.write_uuid(self.uuid)?;
        write.write_string(self.username)?;
        write.write_list::<Property>(self.properties, |p, v| {
            p.write_string(&v.name)?;
            p.write_string(&v.value)?;
            p.write_option(&v.signature, |p, v| p.write_string(v))
        })
    }
}
