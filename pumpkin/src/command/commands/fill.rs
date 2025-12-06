use crate::command::args::block::{
    BlockArgumentConsumer, BlockPredicate, BlockPredicateArgumentConsumer,
};
use crate::command::args::position_block::BlockPosArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArg};
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandError, CommandExecutor, CommandResult, CommandSender};

use pumpkin_data::Block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_world::world::BlockFlags;

const NAMES: [&str; 1] = ["fill"];

const DESCRIPTION: &str = "Fills all or parts of a region with a specific block.";

const ARG_BLOCK: &str = "block";
const ARG_FROM: &str = "from";
const ARG_TO: &str = "to";
const ARG_FILTER: &str = "filter";

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
    /// Replaces all blocks with the new block state, without particles and neighbors update
    Strict,
}

struct Executor(Mode);

fn not_in_filter(filter: &BlockPredicate, old_block: &Block) -> bool {
    match filter {
        BlockPredicate::Tag(tag) => !tag.contains(&old_block.id),
        BlockPredicate::Block(block) => *block != old_block.id,
    }
}

#[expect(clippy::too_many_lines)]
impl CommandExecutor for Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let block = BlockArgumentConsumer::find_arg(args, ARG_BLOCK)?;
            let block_state_id = block.default_state.id;
            let from = BlockPosArgumentConsumer::find_arg(args, ARG_FROM)?;
            let to = BlockPosArgumentConsumer::find_arg(args, ARG_TO)?;
            let option_filter = BlockPredicateArgumentConsumer::find_arg(args, ARG_FILTER)?;
            let mode = self.0;

            let start_x = from.0.x.min(to.0.x);
            let start_y = from.0.y.min(to.0.y);
            let start_z = from.0.z.min(to.0.z);

            let end_x = from.0.x.max(to.0.x);
            let end_y = from.0.y.max(to.0.y);
            let end_z = from.0.z.max(to.0.z);
            // TODO: check isInWorldBounds and throw argument.pos.outofbounds

            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let mut placed_blocks = 0;
            let mut to_update = Vec::new();
            match mode {
                Mode::Destroy => {
                    for x in start_x..=end_x {
                        for y in start_y..=end_y {
                            for z in start_z..=end_z {
                                let block_position = BlockPos(Vector3::new(x, y, z));
                                if let Some(filter) = &option_filter
                                    && not_in_filter(filter, world.get_block(&block_position).await)
                                {
                                    continue;
                                }
                                world
                                    .break_block(
                                        &block_position,
                                        None,
                                        BlockFlags::SKIP_DROPS | BlockFlags::FORCE_STATE,
                                    )
                                    .await;
                                world
                                    .set_block_state(
                                        &block_position,
                                        block_state_id,
                                        BlockFlags::FORCE_STATE,
                                    )
                                    .await;
                                placed_blocks += 1;
                                to_update.push(block_position);
                            }
                        }
                    }
                }
                Mode::Replace => {
                    for x in start_x..=end_x {
                        for y in start_y..=end_y {
                            for z in start_z..=end_z {
                                let block_position = BlockPos(Vector3::new(x, y, z));
                                if let Some(filter) = &option_filter
                                    && not_in_filter(filter, world.get_block(&block_position).await)
                                {
                                    continue;
                                }
                                world
                                    .set_block_state(
                                        &block_position,
                                        block_state_id,
                                        BlockFlags::FORCE_STATE,
                                    )
                                    .await;
                                placed_blocks += 1;
                                to_update.push(block_position);
                            }
                        }
                    }
                }
                Mode::Keep => {
                    for x in start_x..=end_x {
                        for y in start_y..=end_y {
                            for z in start_z..=end_z {
                                let block_position = BlockPos(Vector3::new(x, y, z));
                                let old_state = world.get_block_state(&block_position).await;
                                if old_state.is_air() {
                                    if let Some(filter) = &option_filter
                                        && not_in_filter(
                                            filter,
                                            world.get_block(&block_position).await,
                                        )
                                    {
                                        continue;
                                    }
                                    world
                                        .set_block_state(
                                            &block_position,
                                            block_state_id,
                                            BlockFlags::FORCE_STATE,
                                        )
                                        .await;
                                    placed_blocks += 1;
                                    to_update.push(block_position);
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
                                if let Some(filter) = &option_filter
                                    && not_in_filter(filter, world.get_block(&block_position).await)
                                {
                                    continue;
                                }
                                if is_edge {
                                    world
                                        .set_block_state(
                                            &block_position,
                                            block_state_id,
                                            BlockFlags::FORCE_STATE,
                                        )
                                        .await;
                                } else {
                                    world
                                        .set_block_state(
                                            &block_position,
                                            0,
                                            BlockFlags::FORCE_STATE,
                                        )
                                        .await;
                                }
                                placed_blocks += 1;
                                to_update.push(block_position);
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
                                if !is_edge {
                                    continue;
                                }
                                if let Some(filter) = &option_filter
                                    && not_in_filter(filter, world.get_block(&block_position).await)
                                {
                                    continue;
                                }
                                world
                                    .set_block_state(
                                        &block_position,
                                        block_state_id,
                                        BlockFlags::FORCE_STATE,
                                    )
                                    .await;
                                placed_blocks += 1;
                                to_update.push(block_position);
                            }
                        }
                    }
                }
                Mode::Strict => {
                    for x in start_x..=end_x {
                        for y in start_y..=end_y {
                            for z in start_z..=end_z {
                                let block_position = BlockPos(Vector3::new(x, y, z));
                                if let Some(filter) = &option_filter
                                    && not_in_filter(filter, world.get_block(&block_position).await)
                                {
                                    continue;
                                }
                                world
                                    .set_block_state(
                                        &block_position,
                                        block_state_id,
                                        BlockFlags::SKIP_BLOCK_ADDED_CALLBACK,
                                    )
                                    .await;
                                placed_blocks += 1;
                            }
                        }
                    }
                }
            }

            for i in to_update {
                world.update_neighbors(&i, None).await;
            }

            sender
                .send_message(TextComponent::translate(
                    "commands.fill.success",
                    [TextComponent::text(placed_blocks.to_string())],
                ))
                .await;

            Ok(())
        })
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
                    .then(
                        literal("replace")
                            .then(
                                argument(ARG_FILTER, BlockPredicateArgumentConsumer)
                                    .then(literal("destroy").execute(Executor(Mode::Destroy)))
                                    .then(literal("hollow").execute(Executor(Mode::Hollow)))
                                    .then(literal("keep").execute(Executor(Mode::Keep)))
                                    .then(literal("outline").execute(Executor(Mode::Outline)))
                                    .then(literal("strict").execute(Executor(Mode::Strict)))
                                    .execute(Executor(Mode::Replace)),
                            )
                            .execute(Executor(Mode::Replace)),
                    )
                    .then(literal("strict").execute(Executor(Mode::Strict)))
                    .execute(Executor(Mode::Replace)),
            ),
        ),
    )
}
