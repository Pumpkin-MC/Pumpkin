use crate::Link;
use pumpkin_data::packet::clientbound::CONFIG_SERVER_LINKS;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(CONFIG_SERVER_LINKS)]
pub struct CConfigServerLinks<'a> {
    pub links: &'a [Link<'a>],
}

impl<'a> CConfigServerLinks<'a> {
    #[must_use]
    pub const fn new(links: &'a [Link<'a>]) -> Self {
        Self { links }
    }
}
