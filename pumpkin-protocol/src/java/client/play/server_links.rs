use crate::Link;
use pumpkin_data::packet::clientbound::PLAY_SERVER_LINKS;
use pumpkin_macros::java_packet;
use serde::Serialize;

#[derive(Serialize)]
#[java_packet(PLAY_SERVER_LINKS)]
pub struct CPlayServerLinks<'a> {
    pub links: &'a [Link<'a>],
}

impl<'a> CPlayServerLinks<'a> {
    pub fn new(links: &'a [Link<'a>]) -> Self {
        Self { links }
    }
}
