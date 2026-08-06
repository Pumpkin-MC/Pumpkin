use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::coordinates::block_pos::{
    BlockPosArgumentType, OUT_OF_BOUNDS_ERROR_TYPE,
};
use crate::command::argument_types::resource_key::{BIOME_REGISTRY, ResourceKeyArgument};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use pumpkin_data::translation;
use pumpkin_protocol::java::client::play::CChunkData;
use pumpkin_util::PermissionLvl;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use std::collections::HashMap;

const DESCRIPTION: &str = "Changes biomes of an area.";
const PERMISSION: &str = "minecraft:command.fillbiome";

const MAX_BIOME_BLOCKS: i64 = 32768;

static ERROR_TOOBIG: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_FILLBIOME_TOOBIG,
    translation::java::COMMANDS_FILLBIOME_TOOBIG,
);

struct FillBiomeExecutor {
    has_replace: bool,
}

impl CommandExecutor for FillBiomeExecutor {
    #[expect(clippy::too_many_lines)]
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let from_pos = BlockPosArgumentType::get_block_pos(context, "from")?;
            let to_pos = BlockPosArgumentType::get_block_pos(context, "to")?;

            let world = context.world();
            if !world.is_in_build_limit(from_pos) || !world.is_in_build_limit(to_pos) {
                return Err(OUT_OF_BOUNDS_ERROR_TYPE.create_without_context());
            }

            let min_x = from_pos.0.x.min(to_pos.0.x);
            let max_x = from_pos.0.x.max(to_pos.0.x);
            let min_y = from_pos.0.y.min(to_pos.0.y);
            let max_y = from_pos.0.y.max(to_pos.0.y);
            let min_z = from_pos.0.z.min(to_pos.0.z);
            let max_z = from_pos.0.z.max(to_pos.0.z);

            let biome_min_x = min_x >> 2;
            let biome_max_x = max_x >> 2;
            let biome_min_y = min_y >> 2;
            let biome_max_y = max_y >> 2;
            let biome_min_z = min_z >> 2;
            let biome_max_z = max_z >> 2;

            let volume = (biome_max_x - biome_min_x + 1) as i64
                * (biome_max_y - biome_min_y + 1) as i64
                * (biome_max_z - biome_min_z + 1) as i64;

            if volume > MAX_BIOME_BLOCKS {
                return Err(ERROR_TOOBIG.create_without_context_args_slice(&[
                    TextComponent::text(MAX_BIOME_BLOCKS.to_string()),
                    TextComponent::text(volume.to_string()),
                ]));
            }

            let target_biome = ResourceKeyArgument::get_biome(context, "biome")?;

            let replace_biome = if self.has_replace {
                Some(ResourceKeyArgument::get_biome(context, "replace_biome")?)
            } else {
                None
            };

            let mut chunk_modifications: HashMap<Vector2<i32>, Vec<(usize, usize, usize)>> =
                HashMap::new();
            for y in biome_min_y..=biome_max_y {
                for z in biome_min_z..=biome_max_z {
                    for x in biome_min_x..=biome_max_x {
                        let chunk_pos = Vector2::new(x >> 2, z >> 2);
                        let rel_x = (x & 3) as usize;
                        let rel_z = (z & 3) as usize;
                        let rel_y = (y - (world.min_y >> 2)) as usize;
                        chunk_modifications
                            .entry(chunk_pos)
                            .or_default()
                            .push((rel_x, rel_y, rel_z));
                    }
                }
            }

            let target_biome_id = target_biome.id;
            let replace_biome_id = replace_biome.map(|b| b.id);
            let mut changed_count = 0;

            for (chunk_pos, mods) in chunk_modifications {
                let (has_replaced, count) = world
                    .level
                    .get_or_fetch_chunk(chunk_pos, |chunk| {
                        let mut local_count = 0;
                        let mut modified = false;
                        for &(rel_x, rel_y, rel_z) in &mods {
                            let section_index = rel_y / 4;
                            let scale_y = rel_y % 4;
                            if let Some(current_id) =
                                chunk
                                    .section
                                    .get_noise_biome(section_index, rel_x, scale_y, rel_z)
                            {
                                if let Some(replace_id) = replace_biome_id {
                                    if current_id == replace_id {
                                        chunk.section.set_relative_biome(
                                            rel_x,
                                            rel_y,
                                            rel_z,
                                            target_biome_id,
                                        );
                                        local_count += 1;
                                        modified = true;
                                    }
                                } else {
                                    chunk.section.set_relative_biome(
                                        rel_x,
                                        rel_y,
                                        rel_z,
                                        target_biome_id,
                                    );
                                    local_count += 1;
                                    modified = true;
                                }
                            }
                        }
                        (modified, local_count)
                    })
                    .await;

                if has_replaced {
                    changed_count += count;
                    let chunk = world
                        .level
                        .get_or_fetch_chunk(chunk_pos, std::clone::Clone::clone)
                        .await;
                    world.broadcast_to_chunk_except(chunk_pos, &[], &CChunkData(&chunk));
                }
            }

            let msg = TextComponent::translate_cross(
                translation::java::COMMANDS_FILLBIOME_SUCCESS_COUNT,
                translation::java::COMMANDS_FILLBIOME_SUCCESS_COUNT,
                [TextComponent::text(changed_count.to_string())],
            );
            context.source.send_feedback(msg, true).await;

            Ok(changed_count)
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let builder = command("fillbiome", DESCRIPTION).requires(PERMISSION).then(
        argument("from", BlockPosArgumentType).then(
            argument("to", BlockPosArgumentType).then(
                argument("biome", ResourceKeyArgument(BIOME_REGISTRY.clone()))
                    .executes(FillBiomeExecutor { has_replace: false })
                    .then(
                        argument("replace_biome", ResourceKeyArgument(BIOME_REGISTRY.clone()))
                            .executes(FillBiomeExecutor { has_replace: true }),
                    ),
            ),
        ),
    );

    dispatcher.register(builder);
}
