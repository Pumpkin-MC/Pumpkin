#[allow(clippy::wildcard_imports)]
use super::*;
use crate::entity::vehicle::minecart::MinecartEntity;
use pumpkin_data::translation;
use pumpkin_protocol::java::server::play::SSetCommandMinecart;

impl JavaClient {
    pub fn handle_set_command_minecart(&self, player: &Player, packet: &SSetCommandMinecart<'_>) {
        if !player.is_creative() || player.permission_lvl.load() < PermissionLvl::Two {
            player.send_system_message(&TextComponent::translate(
                translation::java::ADVMODE_NOTALLOWED,
                [],
            ));
            return;
        }

        let world = player.world();
        let Some(entity) = world.get_entity_by_id(packet.entity_id.0) else {
            return;
        };
        let Some(minecart) = entity.cast_any().downcast_ref::<MinecartEntity>() else {
            return;
        };
        let Some(command_block) = minecart.command_block() else {
            return;
        };

        command_block.set_command(packet.command.to_string());
        command_block.set_track_output(packet.track_output);
        if !packet.track_output {
            command_block.set_last_output(None);
        }

        let enabled = world.level_info.load().game_rules.command_blocks_work;
        if enabled {
            command_block.on_updated(&minecart.vehicle.entity);
        }

        if !packet.command.is_empty() {
            let key = if enabled {
                translation::java::ADVMODE_SETCOMMAND_SUCCESS
            } else {
                translation::java::ADVMODE_SETCOMMAND_DISABLED
            };
            player.send_system_message(&TextComponent::translate(
                key,
                [TextComponent::text(packet.command.to_string())],
            ));
        }
    }
}
