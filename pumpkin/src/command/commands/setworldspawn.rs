use std::sync::Arc;

use crate::command::CommandResult;
use crate::command::dispatcher::CommandError::InvalidConsumption;
use crate::command::{
    CommandExecutor, CommandSender,
    args::{
        Arg, ConsumedArgs, position_block::BlockPosArgumentConsumer,
        rotation::RotationArgumentConsumer,
    },
    dispatcher::CommandError,
    tree::{CommandTree, builder::argument},
};
use crate::plugin::world::spawn_change::SpawnChangeEvent;
use crate::server::Server;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::translation;
use pumpkin_util::{math::position::BlockPos, text::TextComponent};

const NAMES: [&str; 1] = ["setworldspawn"];

const DESCRIPTION: &str = "Sets the world spawn point.";

const ARG_BLOCK_POS: &str = "position";

const ARG_ANGLE: &str = "angle";

struct NoArgsWorldSpawnExecutor;

impl CommandExecutor for NoArgsWorldSpawnExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(player) = sender.as_player() else {
                if sender.is_console() {
                    return Err(CommandError::CommandFailed(TextComponent::text(
                        "You must specify a Position!",
                    )));
                }
                return Err(CommandError::CommandFailed(TextComponent::text(
                    "Failed to get Sender as Player!",
                )));
            };
            let block_pos = player.position();
            setworldspawn(sender, server, block_pos.to_block_pos(), 0.0, 0.0).await
        })
    }
}

struct DefaultWorldSpawnExecutor;

impl CommandExecutor for DefaultWorldSpawnExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::BlockPos(block_pos)) = args.get(ARG_BLOCK_POS) else {
                return Err(InvalidConsumption(Some(ARG_BLOCK_POS.into())));
            };

            setworldspawn(sender, server, *block_pos, 0.0, 0.0).await
        })
    }
}

struct AngleWorldSpawnExecutor;

impl CommandExecutor for AngleWorldSpawnExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::BlockPos(block_pos)) = args.get(ARG_BLOCK_POS) else {
                return Err(InvalidConsumption(Some(ARG_BLOCK_POS.into())));
            };

            // Note: Rotation argument is (yaw, is_yaw_relative, pitch, is_pitch_relative)
            // For setworldspawn, we use absolute values only (ignore relative flags)
            let Some(Arg::Rotation(yaw, _, pitch, _)) = args.get(ARG_ANGLE) else {
                return Err(InvalidConsumption(Some(ARG_ANGLE.into())));
            };

            setworldspawn(sender, server, *block_pos, *yaw, *pitch).await
        })
    }
}

async fn setworldspawn(
    sender: &CommandSender,
    server: &Server,
    block_pos: BlockPos,
    yaw: f32,
    pitch: f32,
) -> Result<i32, CommandError> {
    let Some(world) = sender.world() else {
        return Err(CommandError::CommandFailed(TextComponent::text(
            "Failed to get world.",
        )));
    };
    if world.dimension != Dimension::OVERWORLD && world.dimension != Dimension::OVERWORLD_CAVES {
        return Err(CommandError::CommandFailed(TextComponent::translate(
            translation::COMMANDS_SETWORLDSPAWN_FAILURE_NOT_OVERWORLD,
            [],
        )));
    }

    let current_info = server.level_info.load();
    let previous_position = BlockPos::new(
        current_info.spawn_x,
        current_info.spawn_y,
        current_info.spawn_z,
    );
    let mut new_position = block_pos;
    let previous_yaw = current_info.spawn_yaw;
    let previous_pitch = current_info.spawn_pitch;
    let mut new_yaw = yaw;
    let mut new_pitch = pitch;
    let event = SpawnChangeEvent::new(
        world.clone(),
        previous_position,
        previous_yaw,
        previous_pitch,
        new_position,
        new_yaw,
        new_pitch,
    );
    let event = server.plugin_manager.fire(event).await;
    new_position = event.new_position;
    new_yaw = event.new_yaw;
    new_pitch = event.new_pitch;

    let mut new_info = (**current_info).clone();

    new_info.spawn_x = new_position.0.x;
    new_info.spawn_y = new_position.0.y;
    new_info.spawn_z = new_position.0.z;
    new_info.spawn_yaw = new_yaw;
    new_info.spawn_pitch = new_pitch;

    server.level_info.store(Arc::new(new_info));

    sender
        .send_message(TextComponent::translate(
            translation::COMMANDS_SETWORLDSPAWN_SUCCESS,
            [
                TextComponent::text(new_position.0.x.to_string()),
                TextComponent::text(new_position.0.y.to_string()),
                TextComponent::text(new_position.0.z.to_string()),
                TextComponent::text(new_yaw.to_string()),
                TextComponent::text(new_pitch.to_string()),
                TextComponent::text(world.dimension.minecraft_name),
            ],
        ))
        .await;

    Ok(1)
}

#[must_use]
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .execute(NoArgsWorldSpawnExecutor)
        .then(
            argument(ARG_BLOCK_POS, BlockPosArgumentConsumer)
                .execute(DefaultWorldSpawnExecutor)
                .then(
                    argument(ARG_ANGLE, RotationArgumentConsumer).execute(AngleWorldSpawnExecutor),
                ),
        )
}
