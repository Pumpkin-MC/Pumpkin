use crate::Link;
use pumpkin_data::packet::clientbound::play::SERVER_LINKS;
use pumpkin_macros::java_packet;

#[java_packet(SERVER_LINKS)]
pub struct CPlayServerLinks<'a> {
    pub links: &'a [Link<'a>],
}

impl<'a> CPlayServerLinks<'a> {
    #[must_use]
    pub const fn new(links: &'a [Link<'a>]) -> Self {
        Self { links }
    }
}
