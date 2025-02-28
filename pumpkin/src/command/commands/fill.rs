use crate::command::args::block::BlockArgumentConsumer;
use crate::command::args::position_block::BlockPosArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArg};
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandError, CommandExecutor, CommandSender};

use async_trait::async_trait;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["fill"];

const DESCRIPTION: &str = "Fills all or parts of a region with a specific block.";

const ARG_BLOCK: &str = "block";
const ARG_FROM: &str = "from";
const ARG_TO: &str = "to";

#[derive(Clone, Copy, Default)]
enum Mode {
    /// Destroys blocks with particles and item drops
    Destroy,
    /// Leaves only the outer layer of blocks, removes the inner ones (creates a hollow space)
    Hollow,
    /// Only replaces air blocks, keeping non-air blocks unchanged
    Keep,
    /// Like Hollow but doesn't replace inner blocks with air, just the outline
    Outline,
    /// Replaces all blocks with the new block state, without particles
    #[default]
    Replace,
}

struct Executor(Mode);

#[expect(clippy::too_many_lines)]
#[async_trait]
impl CommandExecutor for Executor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let block = BlockArgumentConsumer::find_arg(args, ARG_BLOCK)?;
        let block_state_id = block.default_state_id;
        let from = BlockPosArgumentConsumer::find_arg(args, ARG_FROM)?;
        let to = BlockPosArgumentConsumer::find_arg(args, ARG_TO)?;
        let mode = self.0;

        let start_x = from.0.x.min(to.0.x);
        let start_y = from.0.y.min(to.0.y);
        let start_z = from.0.z.min(to.0.z);

        let end_x = from.0.x.max(to.0.x);
        let end_y = from.0.y.max(to.0.y);
        let end_z = from.0.z.max(to.0.z);

        let world = sender
            .world()
            .await
            .ok_or(CommandError::InvalidRequirement)?;
        let mut placed_blocks = 0;

        match mode {
            Mode::Destroy => {
                for x in start_x..=end_x {
                    for y in start_y..=end_y {
                        for z in start_z..=end_z {
                            let block_position = BlockPos(Vector3 { x, y, z });
                            world
                                .break_block(server, &block_position, None, false)
                                .await;
                            world.set_block_state(&block_position, block_state_id).await;
                            placed_blocks += 1;
                        }
                    }
                }
            }
            Mode::Replace => {
                for x in start_x..=end_x {
                    for y in start_y..=end_y {
                        for z in start_z..=end_z {
                            let block_position = BlockPos(Vector3 { x, y, z });
                            world.set_block_state(&block_position, block_state_id).await;
                            placed_blocks += 1;
                        }
                    }
                }
            }
            Mode::Keep => {
                for x in start_x..=end_x {
                    for y in start_y..=end_y {
                        for z in start_z..=end_z {
                            let block_position = BlockPos(Vector3 { x, y, z });
                            match world.get_block_state(&block_position).await {
                                Ok(old_state) if old_state.air => {
                                    world.set_block_state(&block_position, block_state_id).await;
                                    placed_blocks += 1;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            Mode::Hollow => {
                for x in start_x..=end_x {
                    for y in start_y..=end_y {
                        for z in start_z..=end_z {
                            let block_position = BlockPos(Vector3::new(x, y, z));
                            let is_edge = x == start_x
                                || x == end_x
                                || y == start_y
                                || y == end_y
                                || z == start_z
                                || z == end_z;
                            if is_edge {
                                world.set_block_state(&block_position, block_state_id).await;
                            } else {
                                world.set_block_state(&block_position, 0).await;
                            }
                            placed_blocks += 1;
                        }
                    }
                }
            }
            Mode::Outline => {
                for x in start_x..=end_x {
                    for y in start_y..=end_y {
                        for z in start_z..=end_z {
                            let block_position = BlockPos(Vector3::new(x, y, z));
                            let is_edge = x == start_x
                                || x == end_x
                                || y == start_y
                                || y == end_y
                                || z == start_z
                                || z == end_z;
                            if is_edge {
                                world.set_block_state(&block_position, block_state_id).await;
                                placed_blocks += 1;
                            }
                        }
                    }
                }
            }
        };

        sender
            .send_message(TextComponent::translate(
                "commands.fill.success",
                [TextComponent::text(placed_blocks.to_string())],
            ))
            .await;

        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_FROM, BlockPosArgumentConsumer).then(
            argument(ARG_TO, BlockPosArgumentConsumer).then(
                argument(ARG_BLOCK, BlockArgumentConsumer)
                    .then(literal("destroy").execute(Executor(Mode::Destroy)))
                    .then(literal("hollow").execute(Executor(Mode::Hollow)))
                    .then(literal("keep").execute(Executor(Mode::Keep)))
                    .then(literal("outline").execute(Executor(Mode::Outline)))
                    .then(literal("replace").execute(Executor(Mode::Replace)))
                    .execute(Executor(Mode::Replace)),
            ),
        ),
    )
}
