use std::sync::Arc;

use pumpkin_data::translation;
use pumpkin_protocol::java::client::play::CSetCamera;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_util::{GameMode, PermissionLvl};

use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::EntityBase;

const DESCRIPTION: &str = "Allows a player in spectator mode to spectate a given target entity.";
const PERMISSION: &str = "minecraft:command.spectate";

const ERROR_NOT_PLAYER: CommandErrorType<0> = CommandErrorType::new(
    translation::java::PERMISSIONS_REQUIRES_PLAYER,
    translation::java::PERMISSIONS_REQUIRES_PLAYER,
);

const ERROR_NOT_SPECTATOR: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_SPECTATE_NOT_SPECTATOR,
    translation::java::COMMANDS_SPECTATE_NOT_SPECTATOR,
);

const ERROR_SELF: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_SPECTATE_SELF,
    translation::java::COMMANDS_SPECTATE_SELF,
);

const ERROR_CANNOT_SPECTATE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_SPECTATE_CANNOT_SPECTATE,
    translation::java::COMMANDS_SPECTATE_CANNOT_SPECTATE,
);

struct StopSpectateExecutor;

impl CommandExecutor for StopSpectateExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let player = context
            .source
            .output
            .as_player()
            .ok_or_else(|| ERROR_NOT_PLAYER.create_without_context())?;

        if player.gamemode.load() != GameMode::Spectator {
            let display_name = player.get_display_name();
            return Err(ERROR_NOT_SPECTATOR.create_without_context(display_name));
        }

        player.camera_target_id.store(None);
        player.try_send_client_packet(&CSetCamera::new(player.entity_id().into()));

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_SPECTATE_SUCCESS_STOPPED,
                translation::java::COMMANDS_SPECTATE_SUCCESS_STOPPED,
                [],
            ),
            false,
        );

        Ok(1)
    }
}

struct SpectateTargetExecutor {
    is_self: bool,
}

impl CommandExecutor for SpectateTargetExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let target = EntityArgumentType::get_entity(context, "target")?;
        let target_entity = target.get_entity();
        let target_world = target_entity.world.load_full();

        let players = if self.is_self {
            let player = context
                .source
                .output
                .as_player()
                .ok_or_else(|| ERROR_NOT_PLAYER.create_without_context())?;
            vec![player]
        } else {
            EntityArgumentType::get_players(context, "player")?
        };

        for player in &players {
            if player.gamemode.load() != GameMode::Spectator {
                let display_name = player.get_display_name();
                return Err(ERROR_NOT_SPECTATOR.create_without_context(display_name));
            }

            if target_entity.entity_id == player.entity_id() {
                return Err(ERROR_SELF.create_without_context());
            }

            let player_world = player.world();
            if !Arc::ptr_eq(&target_world, &player_world) {
                let target_name = target.get_display_name();
                return Err(ERROR_CANNOT_SPECTATE.create_without_context(target_name));
            }
        }

        let mut succeeded = 0;
        for player in &players {
            let target_id = target_entity.entity_id;
            player.camera_target_id.store(Some(target_id));
            let pos = target_entity.pos.load();
            let yaw = target_entity.yaw.load();
            let pitch = target_entity.pitch.load();
            let player_world = player.world();
            player.try_send_client_packet(&CSetCamera::new(target_id.into()));
            player.teleport(pos, Some(yaw), Some(pitch), player_world);
            succeeded += 1;
        }

        let target_name = target.get_display_name();
        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_SPECTATE_SUCCESS_STARTED,
                translation::java::COMMANDS_SPECTATE_SUCCESS_STARTED,
                [target_name],
            ),
            false,
        );

        Ok(succeeded)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("spectate", DESCRIPTION)
            .requires(PERMISSION)
            .executes(StopSpectateExecutor)
            .then(
                argument("target", EntityArgumentType::Entity)
                    .executes(SpectateTargetExecutor { is_self: true })
                    .then(
                        argument("player", EntityArgumentType::Players)
                            .executes(SpectateTargetExecutor { is_self: false }),
                    ),
            ),
    );
}
