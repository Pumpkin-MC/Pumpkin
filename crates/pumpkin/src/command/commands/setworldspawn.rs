use std::sync::Arc;

use pumpkin_data::dimension::Dimension;
use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::coordinates::angle::AngleArgumentType;
use crate::command::argument_types::coordinates::block_pos::BlockPosArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::plugin::world::spawn_change::SpawnChangeEvent;

const DESCRIPTION: &str = "Sets the world spawn point.";
const PERMISSION: &str = "minecraft:command.setworldspawn";

const ERROR_NOT_OVERWORLD: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_SETWORLDSPAWN_FAILURE_NOT_OVERWORLD,
    translation::java::COMMANDS_SETWORLDSPAWN_FAILURE_NOT_OVERWORLD,
);

enum SetWorldSpawnMode {
    SelfPos,
    PosOnly,
    PosAndAngle,
}

struct SetWorldSpawnExecutor(SetWorldSpawnMode);

impl CommandExecutor for SetWorldSpawnExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let (pos, angle) = match self.0 {
            SetWorldSpawnMode::SelfPos => {
                let block_pos = BlockPos::floored_v(context.source.position);
                (block_pos, 0.0)
            }
            SetWorldSpawnMode::PosOnly => {
                let block_pos = BlockPosArgumentType::get_block_pos(context, "pos")?;
                (block_pos, 0.0)
            }
            SetWorldSpawnMode::PosAndAngle => {
                let block_pos = BlockPosArgumentType::get_block_pos(context, "pos")?;
                let angle = AngleArgumentType::get(context, "angle")?.get_angle(&context.source);
                (block_pos, angle)
            }
        };

        let world = context.source.world();
        if world.dimension != Dimension::OVERWORLD && world.dimension != Dimension::OVERWORLD_CAVES
        {
            return Err(ERROR_NOT_OVERWORLD.create_without_context());
        }

        let server = context.source.server();
        let current_info = server.level_info.load();
        let previous_position = BlockPos::new(
            current_info.spawn_x,
            current_info.spawn_y,
            current_info.spawn_z,
        );
        let new_position = pos;
        let previous_yaw = current_info.spawn_yaw;
        let previous_pitch = current_info.spawn_pitch;
        let new_yaw = angle;
        let new_pitch = 0.0;
        let mut event = SpawnChangeEvent::new(
            world.clone(),
            previous_position,
            previous_yaw,
            previous_pitch,
            new_position,
            new_yaw,
            new_pitch,
        );
        if let Some(server_arc) = world.server.upgrade() {
            server_arc
                .plugin_manager
                .fire_blocking(&server_arc, &mut event);
        }

        let mut new_info = (**current_info).clone();

        new_info.spawn_x = new_position.0.x;
        new_info.spawn_y = new_position.0.y;
        new_info.spawn_z = new_position.0.z;
        new_info.spawn_yaw = new_yaw;
        new_info.spawn_pitch = new_pitch;

        server.level_info.store(Arc::new(new_info));

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_SETWORLDSPAWN_SUCCESS,
                translation::bedrock::COMMANDS_SETWORLDSPAWN_SUCCESS,
                [
                    TextComponent::text(new_position.0.x.to_string()),
                    TextComponent::text(new_position.0.y.to_string()),
                    TextComponent::text(new_position.0.z.to_string()),
                    TextComponent::text(new_yaw.to_string()),
                    TextComponent::text(new_pitch.to_string()),
                    TextComponent::text(world.dimension.minecraft_name),
                ],
            ),
            true,
        );

        Ok(1)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("setworldspawn", DESCRIPTION)
            .requires(PERMISSION)
            .executes(SetWorldSpawnExecutor(SetWorldSpawnMode::SelfPos))
            .then(
                argument("pos", BlockPosArgumentType)
                    .executes(SetWorldSpawnExecutor(SetWorldSpawnMode::PosOnly))
                    .then(
                        argument("angle", AngleArgumentType)
                            .executes(SetWorldSpawnExecutor(SetWorldSpawnMode::PosAndAngle)),
                    ),
            ),
    );
}
