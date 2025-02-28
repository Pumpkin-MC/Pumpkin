use bytes::BufMut;
use pumpkin_data::packet::clientbound::LOGIN_LOGIN_FINISHED;
use pumpkin_macros::packet;

use crate::{ClientPacket, Property, bytebuf::ByteBufMut};

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
    fn write(&self, bytebuf: &mut impl BufMut) {
        bytebuf.put_uuid(self.uuid);
        bytebuf.put_string(self.username);
        bytebuf.put_list::<Property>(self.properties, |p, v| {
            p.put_string(&v.name);
            p.put_string(&v.value);
            p.put_option(&v.signature, |p, v| p.put_string(v));
        });
    }
}
