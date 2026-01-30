use std::sync::Arc;

use crate::command::args::block::{
    BlockArgumentConsumer, BlockPredicate, BlockPredicateArgumentConsumer,
};
use crate::command::args::position_block::BlockPosArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArg};
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandError, CommandExecutor, CommandResult, CommandSender};
use crate::world::World;

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

enum FillerResult {
    DidNotPlaceBlock = 0,
    PlacedBlock = 1,
    PlacedBlockWithoutUpdate = 2,
}

struct Context {
    block_state_id: u16,
    option_filter: Option<BlockPredicate>,
    world: Arc<World>,
    placed_blocks: i32,
    to_update: Vec<BlockPos>,

    start_x: i32,
    start_y: i32,
    start_z: i32,
    end_x: i32,
    end_y: i32,
    end_z: i32,
}

impl Context {
    /// Checks whether the block position is at the edge of the region stored by this context.
    #[inline]
    const fn is_edge(&self, block_position: BlockPos) -> bool {
        let pos = block_position.0;
        pos.x == self.start_x
            || pos.x == self.end_x
            || pos.y == self.start_y
            || pos.y == self.end_y
            || pos.z == self.start_z
            || pos.z == self.end_z
    }
}

trait Filler {
    async fn execute_for_pos(context: &Context, block_position: BlockPos) -> FillerResult;

    async fn execute_for_region(context: &mut Context) {
        for x in context.start_x..=context.end_x {
            for y in context.start_y..=context.end_y {
                for z in context.start_z..=context.end_z {
                    let block_position = BlockPos(Vector3::new(x, y, z));
                    let filler_result = Self::execute_for_pos(context, block_position).await;
                    match filler_result {
                        FillerResult::PlacedBlock => {
                            context.placed_blocks += 1;
                            context.to_update.push(block_position);
                        }
                        FillerResult::PlacedBlockWithoutUpdate => {
                            context.placed_blocks += 1;
                        }
                        FillerResult::DidNotPlaceBlock => {}
                    }
                }
            }
        }
    }
}

struct DestroyFiller;
impl Filler for DestroyFiller {
    async fn execute_for_pos(context: &Context, block_position: BlockPos) -> FillerResult {
        if let Some(filter) = &context.option_filter
            && not_in_filter(filter, context.world.get_block(&block_position).await)
        {
            return FillerResult::DidNotPlaceBlock;
        }
        context
            .world
            .break_block(
                &block_position,
                None,
                BlockFlags::SKIP_DROPS | BlockFlags::FORCE_STATE,
            )
            .await;
        context
            .world
            .set_block_state(
                &block_position,
                context.block_state_id,
                BlockFlags::FORCE_STATE,
            )
            .await;
        FillerResult::PlacedBlock
    }
}

struct HollowFiller;
impl Filler for HollowFiller {
    async fn execute_for_pos(context: &Context, block_position: BlockPos) -> FillerResult {
        if let Some(filter) = &context.option_filter
            && not_in_filter(filter, context.world.get_block(&block_position).await)
        {
            return FillerResult::DidNotPlaceBlock;
        }
        if context.is_edge(block_position) {
            context
                .world
                .set_block_state(
                    &block_position,
                    context.block_state_id,
                    BlockFlags::FORCE_STATE,
                )
                .await;
        } else {
            context
                .world
                .set_block_state(&block_position, 0, BlockFlags::FORCE_STATE)
                .await;
        }
        FillerResult::PlacedBlock
    }
}

struct KeepFiller;
impl Filler for KeepFiller {
    async fn execute_for_pos(context: &Context, block_position: BlockPos) -> FillerResult {
        let old_state = context.world.get_block_state(&block_position).await;
        if old_state.is_air() {
            if let Some(filter) = &context.option_filter
                && not_in_filter(filter, context.world.get_block(&block_position).await)
            {
                return FillerResult::DidNotPlaceBlock;
            }
            context
                .world
                .set_block_state(
                    &block_position,
                    context.block_state_id,
                    BlockFlags::FORCE_STATE,
                )
                .await;
            FillerResult::PlacedBlock
        } else {
            FillerResult::DidNotPlaceBlock
        }
    }
}

struct OutlineFiller;
impl Filler for OutlineFiller {
    async fn execute_for_pos(context: &Context, block_position: BlockPos) -> FillerResult {
        if !context.is_edge(block_position) {
            return FillerResult::DidNotPlaceBlock;
        }
        if let Some(filter) = &context.option_filter
            && not_in_filter(filter, context.world.get_block(&block_position).await)
        {
            return FillerResult::DidNotPlaceBlock;
        }
        context
            .world
            .set_block_state(
                &block_position,
                context.block_state_id,
                BlockFlags::FORCE_STATE,
            )
            .await;
        FillerResult::PlacedBlock
    }
}

struct ReplaceFiller;
impl Filler for ReplaceFiller {
    async fn execute_for_pos(context: &Context, block_position: BlockPos) -> FillerResult {
        if let Some(filter) = &context.option_filter
            && not_in_filter(filter, context.world.get_block(&block_position).await)
        {
            return FillerResult::DidNotPlaceBlock;
        }
        context
            .world
            .set_block_state(
                &block_position,
                context.block_state_id,
                BlockFlags::FORCE_STATE,
            )
            .await;
        FillerResult::PlacedBlock
    }
}

struct StrictFiller;
impl Filler for StrictFiller {
    async fn execute_for_pos(context: &Context, block_position: BlockPos) -> FillerResult {
        if let Some(filter) = &context.option_filter
            && not_in_filter(filter, context.world.get_block(&block_position).await)
        {
            return FillerResult::DidNotPlaceBlock;
        }
        context
            .world
            .set_block_state(
                &block_position,
                context.block_state_id,
                BlockFlags::SKIP_BLOCK_ADDED_CALLBACK,
            )
            .await;
        FillerResult::PlacedBlockWithoutUpdate
    }
}

impl CommandExecutor for Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let block = BlockArgumentConsumer::find_arg(args, ARG_BLOCK)?;
            let block_state_id = block.default_state.id;
            let from = BlockPosArgumentConsumer::find_arg(args, ARG_FROM)?;
            let to = BlockPosArgumentConsumer::find_arg(args, ARG_TO)?;
            let mode = self.0;

            let mut context = Context {
                block_state_id,
                option_filter: BlockPredicateArgumentConsumer::find_arg(args, ARG_FILTER)?,
                world: sender.world().ok_or(CommandError::InvalidRequirement)?,
                placed_blocks: 0,
                to_update: Vec::new(),

                start_x: from.0.x.min(to.0.x),
                start_y: from.0.y.min(to.0.y),
                start_z: from.0.z.min(to.0.z),

                end_x: from.0.x.max(to.0.x),
                end_y: from.0.y.max(to.0.y),
                end_z: from.0.z.max(to.0.z),
            };

            if !context.world.is_in_build_limit(from) || !context.world.is_in_build_limit(to) {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    "argument.pos.outofbounds",
                    [],
                )));
            }

            let max_block_modifications = {
                let level_info = server.level_info.load();
                level_info.game_rules.max_block_modifications
            };

            let total_blocks = (context.end_x - context.start_x + 1) as i64
                * (context.end_y - context.start_y + 1) as i64
                * (context.end_z - context.start_z + 1) as i64;

            if total_blocks > max_block_modifications {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    "commands.fill.toobig",
                    [
                        TextComponent::text(max_block_modifications.to_string()),
                        TextComponent::text(total_blocks.to_string()),
                    ],
                )));
            }

            match mode {
                Mode::Destroy => DestroyFiller::execute_for_region(&mut context).await,
                Mode::Replace => ReplaceFiller::execute_for_region(&mut context).await,
                Mode::Keep => KeepFiller::execute_for_region(&mut context).await,
                Mode::Hollow => HollowFiller::execute_for_region(&mut context).await,
                Mode::Outline => OutlineFiller::execute_for_region(&mut context).await,
                Mode::Strict => StrictFiller::execute_for_region(&mut context).await,
            }

            for i in context.to_update {
                context.world.update_neighbors(&i, None).await;
            }

            if context.placed_blocks == 0 {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    "commands.fill.failed",
                    [],
                )));
            }

            sender
                .send_message(TextComponent::translate(
                    "commands.fill.success",
                    [TextComponent::text(context.placed_blocks.to_string())],
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
