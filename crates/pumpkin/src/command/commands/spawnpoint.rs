use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::coordinates::angle::AngleArgumentType;
use crate::command::argument_types::coordinates::block_pos::BlockPosArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Sets the spawn point for a player.";
const PERMISSION: &str = "minecraft:command.spawnpoint";

const ERROR_NOT_PLAYER: CommandErrorType<0> = CommandErrorType::new(
    translation::java::PERMISSIONS_REQUIRES_PLAYER,
    translation::java::PERMISSIONS_REQUIRES_PLAYER,
);

enum SpawnpointMode {
    SelfDefault,
    TargetsDefault,
    TargetsPos,
    TargetsPosAngle,
}

struct SpawnpointExecutor(SpawnpointMode);

impl CommandExecutor for SpawnpointExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = match self.0 {
            SpawnpointMode::SelfDefault => {
                let player = context
                    .source
                    .output
                    .as_player()
                    .ok_or_else(|| ERROR_NOT_PLAYER.create_without_context())?;
                vec![player]
            }
            _ => EntityArgumentType::get_players(context, "targets")?,
        };

        for target in &targets {
            let (pos, yaw) = match self.0 {
                SpawnpointMode::SelfDefault | SpawnpointMode::TargetsDefault => {
                    let block_pos = target.position().to_block_pos();
                    let yaw = target.living_entity.entity.yaw.load();
                    (block_pos, yaw)
                }
                SpawnpointMode::TargetsPos => {
                    let block_pos = BlockPosArgumentType::get_block_pos(context, "pos")?;
                    let yaw = target.living_entity.entity.yaw.load();
                    (block_pos, yaw)
                }
                SpawnpointMode::TargetsPosAngle => {
                    let block_pos = BlockPosArgumentType::get_block_pos(context, "pos")?;
                    let yaw = AngleArgumentType::get(context, "angle")?.get_angle(&context.source);
                    (block_pos, yaw)
                }
            };

            let dimension = target.world().dimension.clone();
            target.set_respawn_point(dimension, pos, yaw, 0.0, true);

            context.source.send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_SPAWNPOINT_SUCCESS_SINGLE,
                    translation::bedrock::COMMANDS_SPAWNPOINT_SUCCESS_SINGLE,
                    [
                        TextComponent::text(target.gameprofile.name.clone()),
                        TextComponent::text(pos.0.x.to_string()),
                        TextComponent::text(pos.0.y.to_string()),
                        TextComponent::text(pos.0.z.to_string()),
                    ],
                ),
                true,
            );
        }

        Ok(targets.len() as i32)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("spawnpoint", DESCRIPTION)
            .requires(PERMISSION)
            .executes(SpawnpointExecutor(SpawnpointMode::SelfDefault))
            .then(
                argument("targets", EntityArgumentType::Players)
                    .executes(SpawnpointExecutor(SpawnpointMode::TargetsDefault))
                    .then(
                        argument("pos", BlockPosArgumentType)
                            .executes(SpawnpointExecutor(SpawnpointMode::TargetsPos))
                            .then(
                                argument("angle", AngleArgumentType)
                                    .executes(SpawnpointExecutor(SpawnpointMode::TargetsPosAngle)),
                            ),
                    ),
            ),
    );
}
