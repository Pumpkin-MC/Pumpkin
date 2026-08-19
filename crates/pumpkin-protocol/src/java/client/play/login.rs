use pumpkin_data::packet::clientbound::PLAY_LOGIN;
use pumpkin_util::{resource_location::ResourceLocation, version::JavaMinecraftVersion};

use pumpkin_macros::java_packet;

use crate::{
    ClientPacket, VarInt,
    java::client::play::player_spawn_data::PlayerSpawnData,
    ser::{NetworkWriteExt, WritingError},
};

/// The "Join Game" packet that transitions the client from the Configuration state
/// to the Play state.
///
/// This is one of the largest and most important packets in the protocol. It
/// initializes the player's world view, dimension settings, and local game
/// rules. Once received, the client begins rendering the world.
#[java_packet(PLAY_LOGIN)]
pub struct CLogin<'a> {
    /// The unique ID assigned to the player for the current session.
    pub entity_id: i32,
    pub is_hardcore: bool,
    /// A list of all dimensions present on the server (e.g., overworld, nether, end).
    pub dimension_names: &'a [ResourceLocation],
    pub max_players: VarInt,
    /// The number of chunks the client will render in each direction.
    pub view_distance: VarInt,
    /// The distance at which entities and world ticks are processed.
    pub simulated_distance: VarInt,
    /// If true, hides coordinates and other info from the F3 screen.
    pub reduced_debug_info: bool,
    pub enabled_respawn_screen: bool,
    pub limited_crafting: bool,
    // Spawn info
    pub spawn_data: PlayerSpawnData,
    pub online_mode: bool,
    /// If true, the client will warn the player if they send unsigned chat messages.
    pub enforce_secure_chat: bool,
}

impl<'a> CLogin<'a> {
    #[expect(clippy::too_many_arguments)]
    #[expect(clippy::fn_params_excessive_bools)]
    #[must_use]
    pub const fn new(
        entity_id: i32,
        is_hardcore: bool,
        dimension_names: &'a [ResourceLocation],
        max_players: VarInt,
        view_distance: VarInt,
        simulated_distance: VarInt,
        reduced_debug_info: bool,
        enabled_respawn_screen: bool,
        limited_crafting: bool,
        spawn_data: PlayerSpawnData,
        online_mode: bool,
        enforce_secure_chat: bool,
    ) -> Self {
        Self {
            entity_id,
            is_hardcore,
            dimension_names,
            max_players,
            view_distance,
            simulated_distance,
            reduced_debug_info,
            enabled_respawn_screen,
            limited_crafting,
            spawn_data,
            online_mode,
            enforce_secure_chat,
        }
    }
}

#[must_use]
pub fn build_v1_20_registry_codec(
    version: JavaMinecraftVersion,
) -> pumpkin_nbt::compound::NbtCompound {
    use pumpkin_nbt::compound::NbtCompound;
    use pumpkin_nbt::deserializer::NbtReadHelperJava;
    use pumpkin_nbt::tag::NbtTag;
    use std::io::Cursor;

    let mut root = NbtCompound::new();
    let synced = pumpkin_data::registry::Registry::get_synced(version);

    for reg in synced {
        let reg_name = if let Some(suffix) = reg.registry_id.strip_prefix("minecraft:") {
            suffix
        } else {
            &reg.registry_id
        };
        if version < JavaMinecraftVersion::V_1_20_2
            && !matches!(
                reg_name,
                "dimension_type"
                    | "worldgen/biome"
                    | "chat_type"
                    | "damage_type"
                    | "trim_pattern"
                    | "trim_material"
            )
        {
            continue;
        }

        let mut reg_compound = NbtCompound::new();
        let reg_type = if reg.registry_id.contains(':') {
            reg.registry_id.clone()
        } else {
            format!("minecraft:{}", reg.registry_id)
        };
        reg_compound.put("type", NbtTag::String(reg_type.clone().into()));

        let mut values_list = Vec::new();
        for (i, entry) in reg.registry_entries.iter().enumerate() {
            let mut entry_compound = NbtCompound::new();
            entry_compound.put("name", NbtTag::String(entry.entry_id.clone().into()));
            entry_compound.put("id", NbtTag::Int(i as i32));

            if let Some(ref data) = entry.data {
                let mut cursor = Cursor::new(&data[..]);
                let mut reader = NbtReadHelperJava::new(&mut cursor);
                if let Ok(element_nbt) = pumpkin_nbt::Nbt::read_unnamed(&mut reader) {
                    entry_compound.put("element", NbtTag::Compound(element_nbt.root_tag));
                }
            }
            values_list.push(NbtTag::Compound(entry_compound));
        }

        reg_compound.put("value", NbtTag::List(values_list));
        root.put(&reg_type, NbtTag::Compound(reg_compound));
    }

    root
}

impl ClientPacket for CLogin<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if version < &JavaMinecraftVersion::V_1_20_2 {
            write.write_i32_be(self.entity_id)?;
            write.write_bool(self.is_hardcore)?;
            write.write_u8(self.spawn_data.game_mode)?;
            write.write_i8(self.spawn_data.previous_gamemode)?;
            write.write_list(self.dimension_names, |write, dim| write.write_string(dim))?;

            let registry_codec = build_v1_20_registry_codec(*version);
            let nbt_bytes = pumpkin_nbt::Nbt::new(String::new(), registry_codec).write();
            write.write_all(&nbt_bytes)?;

            write.write_string(self.spawn_data.dimension.minecraft_name)?;
            write.write_string(self.spawn_data.dimension.minecraft_name)?;
            write.write_i64_be(self.spawn_data.hashed_seed)?;
            write.write_var_int(&self.max_players)?;
            write.write_var_int(&self.view_distance)?;
            write.write_var_int(&self.simulated_distance)?;
            write.write_bool(self.reduced_debug_info)?;
            write.write_bool(self.enabled_respawn_screen)?;
            write.write_bool(self.limited_crafting)?;
            write.write_bool(self.spawn_data.debug)?;
            write.write_bool(self.spawn_data.is_flat)?;
            write.write_option(
                &self.spawn_data.death_dimension_name,
                |write, (dim, pos)| {
                    write.write_string(dim)?;
                    write.write_block_pos(pos)?;
                    Ok(())
                },
            )?;
            if version >= &JavaMinecraftVersion::V_1_20 {
                write.write_var_int(&self.spawn_data.portal_cooldown)?;
            }
            return Ok(());
        }

        write.write_i32_be(self.entity_id)?;
        write.write_bool(self.is_hardcore)?;
        if version >= &JavaMinecraftVersion::V_1_16 {
            write.write_list(self.dimension_names, |write, dim| write.write_string(dim))?;
        }
        if version >= &JavaMinecraftVersion::V_1_16 {
            write.write_var_int(&self.max_players)?;
        } else {
            write.write_u8(self.max_players.0 as u8)?;
        }
        if version >= &JavaMinecraftVersion::V_1_14 {
            write.write_var_int(&self.view_distance)?;
        }
        if version >= &JavaMinecraftVersion::V_1_18 {
            write.write_var_int(&self.simulated_distance)?;
        }
        if version >= &JavaMinecraftVersion::V_1_8 {
            write.write_bool(self.reduced_debug_info)?;
        }
        if version >= &JavaMinecraftVersion::V_1_15 {
            write.write_bool(self.enabled_respawn_screen)?;
        }
        if version >= &JavaMinecraftVersion::V_1_19_3 {
            write.write_bool(self.limited_crafting)?;
        }
        self.spawn_data.write_packet_data(&mut write, version)?;
        if version >= &JavaMinecraftVersion::V_26_2 {
            write.write_bool(self.online_mode)?;
        }
        if version >= &JavaMinecraftVersion::V_1_19_1 {
            write.write_bool(self.enforce_secure_chat)?;
        }
        Ok(())
    }
}
