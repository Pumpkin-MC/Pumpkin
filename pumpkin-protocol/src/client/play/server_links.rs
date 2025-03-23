use crate::{Link, VarInt};
use pumpkin_data::packet::clientbound::PLAY_SERVER_LINKS;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_SERVER_LINKS)]
pub struct CPlayServerLinks<'a> {
    links_count: &'a VarInt,
    links: &'a [Link<'a>],
}

impl<'a> CPlayServerLinks<'a> {
    pub fn new(links_count: &'a VarInt, links: &'a [Link<'a>]) -> Self {
        Self { links_count, links }
    }
}
