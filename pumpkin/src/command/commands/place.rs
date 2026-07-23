use pumpkin_data::{Block, BlockStateId, Mirror, Rotation};
use pumpkin_data::structures::{Structure, StructureKeys, StructureType};
use pumpkin_util::identifier::Identifier;
use pumpkin_util::math::block_box::BlockBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::hash_block_pos;
use pumpkin_util::text::TextComponent;
use pumpkin_world::generation::proto_chunk::ProtoChunk;
use pumpkin_world::world::{BlockAccessor, WorldPortalExt};

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::coordinates::block_pos::BlockPosArgumentType;
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::identifier::IdentifierArgumentType;
use crate::command::argument_types::pool::PoolNameArgumentType;
use crate::command::argument_types::structure::StructureNameArgumentType;
use crate::command::argument_types::template::TemplateNameArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::world::block_placer::WorldBlockPlacer;
use pumpkin_world::generation::structure::structures::StructureGeneratorContext;
use pumpkin_world::generation::structure::structures::jigsaw::{
    PoolElementStructurePiece, place_pool_element_templates,
};
use pumpkin_world::generation::structure::structures::jigsaw_placement::{
    DimensionPadding, JigsawPlacement, LiquidSettings, MaxDistance, PoolAliasLookup,
};
use pumpkin_world::generation::structure::template::BlockPlacer;

use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::random::RandomGenerator;
use pumpkin_util::random::legacy_rand::LegacyRand;

const DESCRIPTION: &str = "Places a structure template in the world.";
const PERMISSION: &str = "minecraft:command.place";

static TEMPLATE_NOT_FOUND: CommandErrorType<1> = CommandErrorType::new(
    "commands.place.template.invalid",
    "commands.place.template.invalid",
);
static JIGSAW_FAILED: CommandErrorType<1> = CommandErrorType::new(
    "commands.place.jigsaw.failed",
    "commands.place.jigsaw.failed",
);
static STRUCTURE_INVALID: CommandErrorType<1> = CommandErrorType::new(
    "commands.place.structure.invalid",
    "commands.place.structure.invalid",
);

/// Minimal `WorldPortalExt` implementation for synthetic chunk placement.
struct CommandBlockRegistry;

impl WorldPortalExt for CommandBlockRegistry {
    fn can_place_at(
        &self,
        _block: &pumpkin_data::Block,
        _state: &pumpkin_data::BlockState,
        _block_accessor: &dyn BlockAccessor,
        _block_pos: &BlockPos,
    ) -> bool {
        true
    }

    fn mirror(
        &self,
        block: &pumpkin_data::Block,
        state_id: BlockStateId,
        mirror: Mirror,
    ) -> &'static pumpkin_data::BlockState {
        block.mirror(state_id, mirror)
    }

    fn rotate(
        &self,
        block: &pumpkin_data::Block,
        state_id: BlockStateId,
        rotation: Rotation,
    ) -> &'static pumpkin_data::BlockState {
        block.rotate(state_id, rotation)
    }

    fn spawn_mobs_for_chunk_generation(
        &self,
        _cache: &mut dyn pumpkin_world::generation::proto_chunk::GenerationCache,
        _biome: &'static pumpkin_data::biome::Biome,
        _chunk_x: i32,
        _chunk_z: i32,
    ) {
    }
}

struct PlaceTemplateExecutor;

impl CommandExecutor for PlaceTemplateExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let template_name = context.get_argument::<String>("template")?.clone();

            let block_pos =
                BlockPosArgumentType::get_block_pos(context, "pos").unwrap_or_else(|_| {
                    let p = context.source.position;
                    BlockPos::new(p.x as i32, p.y as i32, p.z as i32)
                });

            let template_name = template_name
                .strip_prefix("minecraft:")
                .unwrap_or(&template_name)
                .to_string();
            let Some(template) =
                pumpkin_world::generation::structure::template::get_template(&template_name)
            else {
                return Err(TEMPLATE_NOT_FOUND
                    .create_without_context(TextComponent::text(template_name.to_string())));
            };

            let mut placer = WorldBlockPlacer::new(context.world());
            pumpkin_world::generation::structure::template::place_template(
                &mut placer,
                &template,
                block_pos.0,
                (0, 0),
                Rotation::None,
                false,
                false,
                &[],
                None,
            );

            placer.finalize().await;
            context
                .world()
                .queue_block_updates(&placer.changed_positions)
                .await;
            context.world().flush_block_updates().await;

            context
                .source
                .send_feedback(
                    TextComponent::translate(
                        "commands.place.template.success",
                        [
                            TextComponent::text(template_name.to_string()),
                            TextComponent::text(block_pos.0.x.to_string()),
                            TextComponent::text(block_pos.0.y.to_string()),
                            TextComponent::text(block_pos.0.z.to_string()),
                        ],
                    ),
                    true,
                )
                .await;

            Ok(1)
        })
    }
}

struct PlaceJigsawExecutor;

impl CommandExecutor for PlaceJigsawExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let pool = context.get_argument::<Identifier>("pool")?.to_string();
            let target = context.get_argument::<Identifier>("target")?.to_string();
            let max_depth = *context.get_argument::<i32>("max_depth")?;

            let block_pos =
                BlockPosArgumentType::get_block_pos(context, "pos").unwrap_or_else(|_| {
                    let p = context.source.position;
                    BlockPos::new(p.x as i32, p.y as i32, p.z as i32)
                });

            let (piece_count, placer) = {
                let seed = hash_block_pos(block_pos.0.x, block_pos.0.y, block_pos.0.z) as u64;
                let random = RandomGenerator::Legacy(LegacyRand::from_seed(seed));
                let mut structure_context = StructureGeneratorContext {
                    seed: seed as i64,
                    chunk_x: 0,
                    chunk_z: 0,
                    random,
                    sea_level: 63,
                    min_y: -64,
                    height_sampler: None,
                    structure_key: None,
                };

                let position = JigsawPlacement::add_pieces(
                    &mut structure_context,
                    &pool,
                    Some(&target),
                    max_depth,
                    block_pos,
                    false,
                    false,
                    &MaxDistance::new(128),
                    &DimensionPadding::ZERO,
                    LiquidSettings::ApplyWaterlog,
                    &PoolAliasLookup,
                )
                .ok_or_else(|| {
                    JIGSAW_FAILED.create_without_context(TextComponent::text(pool.clone()))
                })?;

                let collector = position.collector.lock().unwrap();
                let piece_count = collector.pieces.len();

                let mut placer = WorldBlockPlacer::new(context.world());
                for piece in &collector.pieces {
                    if let Some(jigsaw_piece) =
                        piece.as_any().downcast_ref::<PoolElementStructurePiece>()
                    {
                        place_pool_element_templates(jigsaw_piece, &mut placer, None);
                    }
                }

                (piece_count, placer)
            };

            placer.finalize().await;
            context
                .world()
                .queue_block_updates(&placer.changed_positions)
                .await;
            context.world().flush_block_updates().await;

            context
                .source
                .send_feedback(
                    TextComponent::translate(
                        "commands.place.jigsaw.success",
                        [
                            TextComponent::text(piece_count.to_string()),
                            TextComponent::text(block_pos.0.x.to_string()),
                            TextComponent::text(block_pos.0.y.to_string()),
                            TextComponent::text(block_pos.0.z.to_string()),
                        ],
                    ),
                    true,
                )
                .await;

            Ok(1)
        })
    }
}

struct PlaceStructureExecutor;

impl CommandExecutor for PlaceStructureExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let structure_id = context.get_argument::<Identifier>("structure")?;
            let structure_name = structure_id.to_string();

            let key = StructureKeys::from_name(&structure_name)
                .ok_or_else(|| {
                    STRUCTURE_INVALID
                        .create_without_context(TextComponent::text(structure_name.clone()))
                })?;

            let structure = Structure::get(&key);

            let block_pos = BlockPosArgumentType::get_block_pos(context, "pos")
                .unwrap_or_else(|_| {
                    let p = context.source.position;
                    BlockPos::new(p.x as i32, p.y as i32, p.z as i32)
                });

            let seed = hash_block_pos(block_pos.0.x, block_pos.0.y, block_pos.0.z) as u64;

            let (piece_count, placer) = {
                let world_gen = context.world().level.world_gen.clone();

                if structure.structure_type == StructureType::Jigsaw {
                    let pool = structure.start_pool.ok_or_else(|| {
                        STRUCTURE_INVALID
                            .create_without_context(TextComponent::text(structure_name.clone()))
                    })?;
                    let size = structure.size.ok_or_else(|| {
                        STRUCTURE_INVALID
                            .create_without_context(TextComponent::text(structure_name.clone()))
                    })?;

                    let random = RandomGenerator::Legacy(LegacyRand::from_seed(seed));

                    let position = JigsawPlacement::add_pieces(
                        &mut StructureGeneratorContext {
                            seed: seed as i64,
                            chunk_x: block_pos.0.x >> 4,
                            chunk_z: block_pos.0.z >> 4,
                            random,
                            sea_level: 63,
                            min_y: world_gen.dimension().min_y,
                            height_sampler: None,
                            structure_key: Some(key),
                        },
                        pool,
                        structure.start_jigsaw_name.as_deref(),
                        size,
                        block_pos,
                        structure.use_expansion_hack.unwrap_or(false),
                        structure.project_start_to_heightmap.is_some(),
                        &MaxDistance::new(structure.max_distance_from_center.unwrap_or(128)),
                        &DimensionPadding::ZERO,
                        LiquidSettings::ApplyWaterlog,
                        &PoolAliasLookup,
                    )
                    .ok_or_else(|| {
                        JIGSAW_FAILED
                            .create_without_context(TextComponent::text(structure_name))
                    })?;

                    let collector = position.collector.lock().unwrap();
                    let piece_count = collector.pieces.len();

                    let mut placer = WorldBlockPlacer::new(context.world());
                    for piece in &collector.pieces {
                        if let Some(jigsaw_piece) =
                            piece.as_any().downcast_ref::<PoolElementStructurePiece>()
                        {
                            place_pool_element_templates(jigsaw_piece, &mut placer, None);
                        }
                    }

                    (piece_count, placer)
                } else {
                    let random = RandomGenerator::Legacy(LegacyRand::from_seed(seed));

                    let position = pumpkin_world::generation::structure::generate_structure_position(
                        &key,
                        &structure,
                        StructureGeneratorContext {
                            seed: seed as i64,
                            chunk_x: block_pos.0.x >> 4,
                            chunk_z: block_pos.0.z >> 4,
                            random,
                            sea_level: 63,
                            min_y: world_gen.dimension().min_y,
                            height_sampler: None,
                            structure_key: Some(key),
                        },
                    )
                    .ok_or_else(|| {
                        STRUCTURE_INVALID
                            .create_without_context(TextComponent::text(structure_name))
                    })?;

                    let mut collector = position.collector.lock().unwrap();
                    let piece_count = collector.pieces.len();

                    let mut placer = WorldBlockPlacer::new(context.world());

                    for piece in &collector.pieces {
                        if let Some(jigsaw_piece) =
                            piece.as_any().downcast_ref::<PoolElementStructurePiece>()
                        {
                            place_pool_element_templates(jigsaw_piece, &mut placer, None);
                        }
                    }

                    let has_non_jigsaw = collector
                        .pieces
                        .iter()
                        .any(|p| p.as_any().downcast_ref::<PoolElementStructurePiece>().is_none());

                    if has_non_jigsaw {
                        let reg = CommandBlockRegistry;

                        let mut min_x = i32::MAX;
                        let mut min_y = i32::MAX;
                        let mut min_z = i32::MAX;
                        let mut max_x = i32::MIN;
                        let mut max_y = i32::MIN;
                        let mut max_z = i32::MIN;

                        for piece in &collector.pieces {
                            if piece.as_any().downcast_ref::<PoolElementStructurePiece>().is_none() {
                                let bb = piece.bounding_box();
                                if bb.min.x < min_x { min_x = bb.min.x; }
                                if bb.min.y < min_y { min_y = bb.min.y; }
                                if bb.min.z < min_z { min_z = bb.min.z; }
                                if bb.max.x > max_x { max_x = bb.max.x; }
                                if bb.max.y > max_y { max_y = bb.max.y; }
                                if bb.max.z > max_z { max_z = bb.max.z; }
                            }
                        }

                        let start_cx = min_x >> 4;
                        let end_cx = max_x >> 4;
                        let start_cz = min_z >> 4;
                        let end_cz = max_z >> 4;

                        for cx in start_cx..=end_cx {
                            for cz in start_cz..=end_cz {
                                let mut chunk = ProtoChunk::new(cx, cz, &world_gen);
                                let chunk_min_y = chunk.bottom_y() as i32;
                                let chunk_height = chunk.height() as i32;
                                let ground_y = (block_pos.0.y - 1)
                                    .clamp(chunk_min_y, chunk_min_y + chunk_height - 1);

                                // Pre-seed heightmaps so get_top_y returns a reasonable value
                                let ground = ground_y as i16;
                                chunk.flat_surface_height_map = [ground; 256];
                                chunk.flat_ocean_floor_height_map = [ground; 256];
                                chunk.flat_motion_blocking_height_map = [ground; 256];
                                chunk.flat_motion_blocking_no_leaves_height_map = [ground; 256];

                                // Place a thin stone floor so pieces that scan blocks find solid ground
                                for x in 0..16i32 {
                                    for z in 0..16i32 {
                                        chunk.set_block_state(
                                            x, ground_y, z, Block::STONE.default_state,
                                        );
                                    }
                                }

                                let chunk_box = BlockBox::new(
                                    cx << 4,
                                    chunk_min_y,
                                    cz << 4,
                                    (cx << 4) + 15,
                                    chunk_min_y + chunk_height - 1,
                                    (cz << 4) + 15,
                                );

                                // Snapshot the pre-populated state so we only send
                                // the structure's own blocks to the world
                                let mut snapshot =
                                    Vec::with_capacity((16 * chunk_height * 16) as usize);
                                for x in 0..16 {
                                    for y in chunk_min_y..chunk_min_y + chunk_height {
                                        for z in 0..16 {
                                            snapshot.push(chunk.get_block_state(&Vector3::new(
                                                (cx << 4) + x,
                                                y,
                                                (cz << 4) + z,
                                            )));
                                        }
                                    }
                                }

                                let chunk_seed = (cx as i64)
                                    .wrapping_mul(341873128712)
                                    .wrapping_add((cz as i64).wrapping_mul(132897987541))
                                    .wrapping_add(seed as i64);
                                let mut rng = RandomGenerator::Legacy(LegacyRand::from_seed(
                                    chunk_seed as u64,
                                ));

                                for piece in &mut collector.pieces {
                                    if piece.bounding_box().intersects(&chunk_box) {
                                        piece.place(
                                            &mut chunk,
                                            &reg,
                                            &mut rng,
                                            seed as i64,
                                            &chunk_box,
                                        );
                                    }
                                }

                                // Delta: only send blocks changed by the structure
                                let mut snapshot_idx = 0usize;
                                for x in 0..16 {
                                    for y in chunk_min_y..chunk_min_y + chunk_height {
                                        for z in 0..16 {
                                            let pos = Vector3::new(
                                                (cx << 4) + x,
                                                y,
                                                (cz << 4) + z,
                                            );
                                            let new_id = chunk.get_block_state(&pos);
                                            if new_id != snapshot[snapshot_idx] {
                                                placer.set_block_state(
                                                    &pos,
                                                    new_id.to_state(),
                                                );
                                            }
                                            snapshot_idx += 1;
                                        }
                                    }
                                }

                                for nbt in chunk.pending_block_entities.drain(..) {
                                    placer.block_entity_nbts.push(nbt);
                                }
                            }
                        }
                    }

                    (piece_count, placer)
                }
            };

            placer.finalize().await;
            context
                .world()
                .queue_block_updates(&placer.changed_positions)
                .await;
            context.world().flush_block_updates().await;

            context
                .source
                .send_feedback(
                    TextComponent::translate(
                        "commands.place.structure.success",
                        [
                            TextComponent::text(piece_count.to_string()),
                            TextComponent::text(block_pos.0.x.to_string()),
                            TextComponent::text(block_pos.0.y.to_string()),
                            TextComponent::text(block_pos.0.z.to_string()),
                        ],
                    ),
                    true,
                )
                .await;

            Ok(1)
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("place", DESCRIPTION)
            .requires(PERMISSION)
            .then(
                literal("template").then(
                    argument("template", TemplateNameArgumentType)
                        .executes(PlaceTemplateExecutor)
                        .then(
                            argument("pos", BlockPosArgumentType).executes(PlaceTemplateExecutor),
                        ),
                ),
            )
            .then(
                literal("jigsaw").then(
                    argument("pool", PoolNameArgumentType).then(
                        argument("target", IdentifierArgumentType).then(
                            argument("max_depth", IntegerArgumentType::new(1, 20))
                                .executes(PlaceJigsawExecutor)
                                .then(
                                    argument("pos", BlockPosArgumentType)
                                        .executes(PlaceJigsawExecutor),
                                ),
                        ),
                    ),
                ),
            )
            .then(
                literal("structure").then(
                    argument("structure", StructureNameArgumentType)
                        .executes(PlaceStructureExecutor)
                        .then(
                            argument("pos", BlockPosArgumentType)
                                .executes(PlaceStructureExecutor),
                        ),
                ),
            ),
    );
}
