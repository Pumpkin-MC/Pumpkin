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
use crate::server::Server;
use pumpkin_data::dimension::Dimension;
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

            let Some(Arg::Rotation(pitch, yaw)) = args.get(ARG_ANGLE) else {
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
) -> Result<(), CommandError> {
    let Some(world) = sender.world() else {
        return Err(CommandError::CommandFailed(TextComponent::text(
            "Failed to get world.",
        )));
    };
    if world.dimension != Dimension::OVERWORLD && world.dimension != Dimension::OVERWORLD_CAVES {
        sender
            .send_message(TextComponent::translate(
                "commands.setworldspawn.failure.not_overworld",
                [],
            ))
            .await;
        return Ok(());
    }

    let mut level_info_guard = server.level_info.write().await;
    level_info_guard.spawn_x = block_pos.0.x;
    level_info_guard.spawn_y = block_pos.0.y;
    level_info_guard.spawn_z = block_pos.0.z;

    level_info_guard.spawn_yaw = yaw;
    level_info_guard.spawn_pitch = pitch;

    drop(level_info_guard);

    sender
        .send_message(TextComponent::translate(
            "commands.setworldspawn.success",
            [
                TextComponent::text(block_pos.0.x.to_string()),
                TextComponent::text(block_pos.0.y.to_string()),
                TextComponent::text(block_pos.0.z.to_string()),
                TextComponent::text(yaw.to_string()),
                TextComponent::text(pitch.to_string()),
                TextComponent::text(world.dimension.minecraft_name),
            ],
        ))
        .await;

    Ok(())
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
