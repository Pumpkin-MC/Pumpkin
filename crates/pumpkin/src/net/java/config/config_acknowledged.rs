#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_config_acknowledged(&self, server: &Server) -> PacketHandlerResult {
        debug!("Handling config acknowledgement");
        self.connection_state.store(ConnectionState::Play);

        let profile = self.gameprofile.clone();
        let address = self.address;

        if let Some(reason) = can_not_join(&profile, &address, server).await {
            self.kick(reason).await;
            return PacketHandlerResult::Stop;
        }

        let config = self.config.load();
        PacketHandlerResult::ReadyToPlay(profile, (**config).clone())
    }
}

pub(crate) fn build_dimension_nbt(dim: &pumpkin_data::dimension::Dimension) -> Vec<u8> {
    let mut compound = pumpkin_nbt::compound::NbtCompound::new();
    compound.put_float("ambient_light", dim.ambient_light);
    compound.put_int("height", dim.height);
    compound.put_int("logical_height", dim.logical_height);
    compound.put_int("min_y", dim.min_y);
    compound.put_string("infiniburn", dim.infiniburn.to_string());
    compound.put_int(
        "monster_spawn_block_light_limit",
        dim.monster_spawn_block_light_limit as i32,
    );
    compound.put_double("coordinate_scale", dim.coordinate_scale);
    compound.put_byte("has_skylight", i8::from(dim.has_skylight));
    compound.put_byte("has_ceiling", i8::from(dim.has_ceiling));
    compound.put_byte("ultrawarm", i8::from(dim.id == 3));
    compound.put_byte("natural", i8::from(dim.id == 0 || dim.id == 1));
    compound.put_byte("piglin_safe", i8::from(dim.id == 3));
    compound.put_byte("respawn_anchor_works", i8::from(dim.id == 3));
    compound.put_byte("bed_works", i8::from(dim.id == 0 || dim.id == 1));
    compound.put_byte("has_raids", i8::from(dim.id == 0 || dim.id == 1));
    compound.put_string("effects", dim.minecraft_name.to_string());

    let mut monster_spawn = pumpkin_nbt::compound::NbtCompound::new();
    monster_spawn.put_string("type", "minecraft:uniform".to_string());
    let mut value = pumpkin_nbt::compound::NbtCompound::new();
    value.put_int("min_inclusive", 0);
    value.put_int("max_inclusive", 7);
    monster_spawn.put_compound("value", value);
    compound.put_compound("monster_spawn_light_level", monster_spawn);

    pumpkin_nbt::Nbt::from(compound).write().to_vec()
}
