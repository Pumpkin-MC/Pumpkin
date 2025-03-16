use bytes::BufMut;
use pumpkin_data::{
    fluid::Fluid,
    packet::clientbound::CONFIG_UPDATE_TAGS,
    tag::{RegistryKey, get_registry_key_tags},
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

            let values = get_registry_key_tags(registry_key);
            p.put_var_int(&VarInt::from(values.len() as i32));
            for (key, values) in values.iter() {
                // This is technically an `Identifier` but same thing
                p.put_string_len(key, u16::MAX as usize);
                p.put_list(values, |p, string_id| {
                    let id = match registry_key {
                        RegistryKey::Block => registry::get_block(string_id).unwrap().id as i32,
                        RegistryKey::Fluid => Fluid::ident_to_fluid_id(string_id).unwrap() as i32,
                        _ => unimplemented!(),
                    };

                    p.put_var_int(&VarInt::from(id));
                });
            }
        });
    }
}
