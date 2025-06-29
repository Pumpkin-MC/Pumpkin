use pumpkin_data::packet::clientbound::LOGIN_LOGIN_FINISHED;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::Property;

#[derive(Serialize)]
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
