use std::sync::Arc;

use pumpkin_data::translation;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_world::world::BlockFlags;

use crate::command::args::block::BlockPredicateArgumentConsumer;
use crate::command::args::position_block::BlockPosArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArg};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};
use crate::world::World;

const NAMES: [&str; 1] = ["clone"];

const DESCRIPTION: &str = "Copies blocks from one region to another.";

const ARG_BEGIN: &str = "begin";
const ARG_END: &str = "end";
const ARG_DESTINATION: &str = "destination";
const ARG_FILTER: &str = "filter";

#[derive(Clone, Copy, Default)]
enum MaskMode {
    #[default]
    Replace,
    Masked,
    Filtered,
}

#[derive(Clone, Copy, Default)]
enum CloneMode {
    Force,
    Move,
    #[default]
    Normal,
}

struct CloneExecutor {
    mask_mode: MaskMode,
    clone_mode: CloneMode,
}

struct StoredBlock {
    pos: BlockPos,
    state_id: u16,
}

const fn regions_overlap(src_min: Vector3<i32>, src_max: Vector3<i32>, dst: Vector3<i32>) -> bool {
    let size = Vector3::new(
        src_max.x - src_min.x,
        src_max.y - src_min.y,
        src_max.z - src_min.z,
    );
    let dst_max = Vector3::new(dst.x + size.x, dst.y + size.y, dst.z + size.z);

    src_min.x <= dst_max.x
        && src_max.x >= dst.x
        && src_min.y <= dst_max.y
        && src_max.y >= dst.y
        && src_min.z <= dst_max.z
        && src_max.z >= dst.z
}

async fn place_blocks(
    world: &Arc<World>,
    blocks: &[StoredBlock],
    offset: Vector3<i32>,
    clone_mode: CloneMode,
) -> i32 {
    let mut placed = 0i32;
    for block in blocks {
        let dst_pos = BlockPos(Vector3::new(
            block.pos.0.x + offset.x,
            block.pos.0.y + offset.y,
            block.pos.0.z + offset.z,
        ));
        if world.is_in_build_limit(dst_pos) {
            world
                .set_block_state(&dst_pos, block.state_id, BlockFlags::NOTIFY_ALL)
                .await;
            placed += 1;
        }
    }

    if matches!(clone_mode, CloneMode::Move) {
        for block in blocks {
            world
                .set_block_state(&block.pos, 0, BlockFlags::NOTIFY_ALL)
                .await;
        }
    }

    placed
}

async fn collect_blocks(
    world: &Arc<World>,
    src_min: Vector3<i32>,
    src_max: Vector3<i32>,
    mask_mode: MaskMode,
    filter: Option<&crate::command::args::block::BlockPredicate>,
) -> Vec<StoredBlock> {
    let mut blocks = Vec::new();
    for x in src_min.x..=src_max.x {
        for y in src_min.y..=src_max.y {
            for z in src_min.z..=src_max.z {
                let pos = BlockPos(Vector3::new(x, y, z));
                let state_id = world.get_block_state_id(&pos).await;

                let include = match mask_mode {
                    MaskMode::Replace => true,
                    MaskMode::Masked => !pumpkin_data::block_properties::is_air(state_id),
                    MaskMode::Filtered => {
                        if let Some(filter) = filter {
                            let block = world.get_block(&pos).await;
                            match filter {
                                crate::command::args::block::BlockPredicate::Tag(tag) => {
                                    tag.contains(&block.id)
                                }
                                crate::command::args::block::BlockPredicate::Block(id) => {
                                    *id == block.id
                                }
                            }
                        } else {
                            true
                        }
                    }
                };

                if include {
                    blocks.push(StoredBlock { pos, state_id });
                }
            }
        }
    }
    blocks
}

impl CommandExecutor for CloneExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        let mask_mode = self.mask_mode;
        let clone_mode = self.clone_mode;
        Box::pin(async move {
            let begin = BlockPosArgumentConsumer::find_arg(args, ARG_BEGIN)?;
            let end = BlockPosArgumentConsumer::find_arg(args, ARG_END)?;
            let destination = BlockPosArgumentConsumer::find_arg(args, ARG_DESTINATION)?;

            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;

            if !world.is_in_build_limit(begin)
                || !world.is_in_build_limit(end)
                || !world.is_in_build_limit(destination)
            {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    "argument.pos.outofbounds",
                    [],
                )));
            }

            let src_min = Vector3::new(
                begin.0.x.min(end.0.x),
                begin.0.y.min(end.0.y),
                begin.0.z.min(end.0.z),
            );
            let src_max = Vector3::new(
                begin.0.x.max(end.0.x),
                begin.0.y.max(end.0.y),
                begin.0.z.max(end.0.z),
            );

            let total_blocks = (src_max.x - src_min.x + 1) as i64
                * (src_max.y - src_min.y + 1) as i64
                * (src_max.z - src_min.z + 1) as i64;

            let max_block_modifications = {
                let level_info = server.level_info.load();
                level_info.game_rules.max_block_modifications
            };

            if total_blocks > max_block_modifications {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_CLONE_TOOBIG,
                    [
                        TextComponent::text(max_block_modifications.to_string()),
                        TextComponent::text(total_blocks.to_string()),
                    ],
                )));
            }

            // Check overlap for normal mode
            if matches!(clone_mode, CloneMode::Normal)
                && regions_overlap(src_min, src_max, destination.0)
            {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_CLONE_OVERLAP,
                    [],
                )));
            }

            let filter = if matches!(mask_mode, MaskMode::Filtered) {
                BlockPredicateArgumentConsumer::find_arg(args, ARG_FILTER)?
            } else {
                None
            };

            // Collect source blocks
            let blocks = collect_blocks(&world, src_min, src_max, mask_mode, filter.as_ref()).await;

            if blocks.is_empty() {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_CLONE_FAILED,
                    [],
                )));
            }

            let offset = Vector3::new(
                destination.0.x - src_min.x,
                destination.0.y - src_min.y,
                destination.0.z - src_min.z,
            );

            let placed = place_blocks(&world, &blocks, offset, clone_mode).await;

            if placed == 0 {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_CLONE_FAILED,
                    [],
                )));
            }

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_CLONE_SUCCESS,
                    [TextComponent::text(placed.to_string())],
                ))
                .await;
            Ok(placed)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_BEGIN, BlockPosArgumentConsumer).then(
            argument(ARG_END, BlockPosArgumentConsumer).then(
                argument(ARG_DESTINATION, BlockPosArgumentConsumer)
                    .then(
                        literal("replace")
                            .then(literal("force").execute(CloneExecutor {
                                mask_mode: MaskMode::Replace,
                                clone_mode: CloneMode::Force,
                            }))
                            .then(literal("move").execute(CloneExecutor {
                                mask_mode: MaskMode::Replace,
                                clone_mode: CloneMode::Move,
                            }))
                            .then(literal("normal").execute(CloneExecutor {
                                mask_mode: MaskMode::Replace,
                                clone_mode: CloneMode::Normal,
                            }))
                            .execute(CloneExecutor {
                                mask_mode: MaskMode::Replace,
                                clone_mode: CloneMode::Normal,
                            }),
                    )
                    .then(
                        literal("masked")
                            .then(literal("force").execute(CloneExecutor {
                                mask_mode: MaskMode::Masked,
                                clone_mode: CloneMode::Force,
                            }))
                            .then(literal("move").execute(CloneExecutor {
                                mask_mode: MaskMode::Masked,
                                clone_mode: CloneMode::Move,
                            }))
                            .then(literal("normal").execute(CloneExecutor {
                                mask_mode: MaskMode::Masked,
                                clone_mode: CloneMode::Normal,
                            }))
                            .execute(CloneExecutor {
                                mask_mode: MaskMode::Masked,
                                clone_mode: CloneMode::Normal,
                            }),
                    )
                    .then(
                        literal("filtered").then(
                            argument(ARG_FILTER, BlockPredicateArgumentConsumer)
                                .then(literal("force").execute(CloneExecutor {
                                    mask_mode: MaskMode::Filtered,
                                    clone_mode: CloneMode::Force,
                                }))
                                .then(literal("move").execute(CloneExecutor {
                                    mask_mode: MaskMode::Filtered,
                                    clone_mode: CloneMode::Move,
                                }))
                                .then(literal("normal").execute(CloneExecutor {
                                    mask_mode: MaskMode::Filtered,
                                    clone_mode: CloneMode::Normal,
                                }))
                                .execute(CloneExecutor {
                                    mask_mode: MaskMode::Filtered,
                                    clone_mode: CloneMode::Normal,
                                }),
                        ),
                    )
                    .execute(CloneExecutor {
                        mask_mode: MaskMode::Replace,
                        clone_mode: CloneMode::Normal,
                    }),
            ),
        ),
    )
}
