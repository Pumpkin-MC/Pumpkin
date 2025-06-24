use async_trait::async_trait;
use pumpkin_registry::VanillaDimensionType;
use pumpkin_util::text::TextComponent;
use crate::command::dispatcher::CommandError::InvalidConsumption;
use crate::command::{args::{position_block::BlockPosArgumentConsumer, rotation::RotationArgumentConsumer, Arg, ConsumedArgs}, dispatcher::CommandError, tree::{builder::argument, CommandTree}, CommandExecutor, CommandSender};

const NAMES: [&str; 1] = ["setworldspawn"];

const DESCRIPTION: &str = "Sets the world spawn point.";

const ARG_BLOCK_POS: &str = "position";

const ARG_ANGLE: &str = "angle";

struct DefaultWorldSpawnExecutor;

#[async_trait]
impl CommandExecutor for DefaultWorldSpawnExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::BlockPos(block_pos)) = args.get(ARG_BLOCK_POS) else {
            return Err(InvalidConsumption(Some(ARG_BLOCK_POS.into())));
        };
        
        let Some(world) = sender.world().await else {
            return Err(CommandError::GeneralCommandIssue("Failed to get world.".into()))
        };
        
        let level_info_guard = world.level_info.write().await;
        level_info_guard.spawn_x = block_pos.0.x;
        level_info_guard.spawn_y = block_pos.0.y;
        level_info_guard.spawn_z = block_pos.0.z;
        
        Ok(())
    }
}

struct AngleWorldSpawnExecutor;

#[async_trait]
impl CommandExecutor for AngleWorldSpawnExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::BlockPos(block_pos)) = args.get(ARG_BLOCK_POS) else {
            return Err(InvalidConsumption(Some(ARG_BLOCK_POS.into())));
        };
        
        let Some(Arg::Rotation(_, yaw)) = args.get(ARG_ANGLE) else {
            return Err(InvalidConsumption(Some(ARG_ANGLE.into())));
        };
        
        let Some(world) = sender.world().await else {
            return Err(CommandError::GeneralCommandIssue("Failed to get world.".into()))
        };
        
        if world.dimension_type != VanillaDimensionType::Overworld && world.dimension_type != VanillaDimensionType::OverworldCaves {
            sender.send_message(TextComponent::translate("commands.setworldspawn.failure.not_overworld", []));
            return Ok(());
        }
        
        let mut level_info_guard = world.level_info.write().await;
        level_info_guard.spawn_x = block_pos.0.x;
        level_info_guard.spawn_y = block_pos.0.y;
        level_info_guard.spawn_z = block_pos.0.z;
        
        level_info_guard.spawn_angle = *yaw;
        
        drop(level_info_guard);
        
        sender.send_message(TextComponent::translate("commands.setworldspawn.success", [
            TextComponent::text(block_pos.0.x.to_string()),
            TextComponent::text(block_pos.0.y.to_string()),
            TextComponent::text(block_pos.0.z.to_string()),
        ]));
        
        Ok(())
    }
}

#[must_use]
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_BLOCK_POS, BlockPosArgumentConsumer).execute(DefaultWorldSpawnExecutor)
            .then(argument(ARG_ANGLE, RotationArgumentConsumer)))
}
