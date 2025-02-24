use bytes::BufMut;
use pumpkin_data::{
    fluid::Fluid,
    packet::clientbound::CONFIG_UPDATE_TAGS,
    tag::{RegistryKey, TAGS},
};
use pumpkin_macros::packet;
use pumpkin_world::block::registry;

use crate::{
    ClientPacket,
    bytebuf::ByteBufMut,
    codec::{identifier::Identifier, var_int::VarInt},
};

#[packet(CONFIG_UPDATE_TAGS)]
pub struct CUpdateTags<'a> {
    tags: &'a [pumpkin_data::tag::RegistryKey],
}

impl<'a> CUpdateTags<'a> {
    pub fn new(tags: &'a [pumpkin_data::tag::RegistryKey]) -> Self {
        Self { tags }
    }
}

impl ClientPacket for CUpdateTags<'_> {
    fn write(&self, bytebuf: &mut impl BufMut) {
        bytebuf.put_list(self.tags, |p, registry_key| {
            p.put_identifier(&Identifier::vanilla(registry_key.identifier_string()));

            let values = TAGS.get(registry_key).unwrap();
            p.put_var_int(&VarInt::from(values.len() as i32));
            for (key, values) in values.iter() {
                // This is technically a Identifier but same thing
                p.put_string_len(key, u16::MAX as usize);
                p.put_list(values, |p, v| {
                    if let Some(string_id) = v {
                        let id = match registry_key {
                            RegistryKey::Block => registry::get_block(string_id).unwrap().id as i32,
                            RegistryKey::Fluid => {
                                Fluid::ident_to_fluid_id(string_id).unwrap() as i32
                            }
                            _ => unimplemented!(),
                        };

                        p.put_var_int(&VarInt::from(id));
                    }
                });
            }
        });
    }
}
