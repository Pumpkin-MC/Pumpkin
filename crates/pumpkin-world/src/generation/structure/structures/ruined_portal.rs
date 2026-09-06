use std::sync::Arc;

use pumpkin_data::{Block, Mirror, Rotation, structures::StructureKeys};
use pumpkin_util::{
    HeightMap,
    math::{block_box::BlockBox, position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl, hash_block_pos, legacy_rand::LegacyRand},
};

use crate::{
    ProtoChunk,
    generation::{
        proto_chunk::GenerationCache,
        structure::{
            piece::StructurePieceType,
            structures::{
                HeightSampler, StructureGenerator, StructureGeneratorContext, StructurePiece,
                StructurePieceBase, StructurePiecesCollector, StructurePosition, WorldPortalExt,
            },
            template::{
                BlockStateResolver, PaletteEntry, StructurePlaceSettings, StructureTemplate,
                get_template,
                processor::{
                    IgnoredBlock, PosRuleTest, ProcessorContext, ProcessorRule, RuleTest,
                    StructureProcessor,
                },
            },
        },
    },
};

const PORTALS: &[&str] = &[
    "ruined_portal/portal_1",
    "ruined_portal/portal_2",
    "ruined_portal/portal_3",
    "ruined_portal/portal_4",
    "ruined_portal/portal_5",
    "ruined_portal/portal_6",
    "ruined_portal/portal_7",
    "ruined_portal/portal_8",
    "ruined_portal/portal_9",
    "ruined_portal/portal_10",
];

const GIANT_PORTALS: &[&str] = &[
    "ruined_portal/giant_portal_1",
    "ruined_portal/giant_portal_2",
    "ruined_portal/giant_portal_3",
];

const NETHERRACK_PROBABILITY_BY_DISTANCE: &[f32] = &[
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.9, 0.9, 0.8, 0.7, 0.6, 0.4, 0.2,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerticalPlacement {
    OnLandSurface,
    PartlyBuried,
    OnOceanFloor,
    InMountain,
    Underground,
    InNether,
}

impl VerticalPlacement {
    #[must_use]
    pub const fn get_heightmap_type(self) -> HeightMap {
        match self {
            Self::OnOceanFloor => HeightMap::OceanFloorWg,
            _ => HeightMap::WorldSurfaceWg,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RuinedPortalProperties {
    pub cold: bool,
    pub mossiness: f32,
    pub air_pocket: bool,
    pub overgrown: bool,
    pub vines: bool,
    pub replace_with_blackstone: bool,
}

/// One entry of a `minecraft:ruined_portal` structure's `setups` list.
#[derive(Clone, Copy)]
pub struct RuinedPortalSetup {
    pub placement: VerticalPlacement,
    pub air_pocket_probability: f32,
    pub mossiness: f32,
    pub overgrown: bool,
    pub vines: bool,
    pub can_be_cold: bool,
    pub replace_with_blackstone: bool,
    pub weight: f32,
}

const DESERT_SETUPS: &[RuinedPortalSetup] = &[RuinedPortalSetup {
    placement: VerticalPlacement::PartlyBuried,
    air_pocket_probability: 0.0,
    mossiness: 0.0,
    overgrown: false,
    vines: false,
    can_be_cold: false,
    replace_with_blackstone: false,
    weight: 1.0,
}];
const JUNGLE_SETUPS: &[RuinedPortalSetup] = &[RuinedPortalSetup {
    placement: VerticalPlacement::OnLandSurface,
    air_pocket_probability: 0.5,
    mossiness: 0.8,
    overgrown: true,
    vines: true,
    can_be_cold: false,
    replace_with_blackstone: false,
    weight: 1.0,
}];
const SWAMP_SETUPS: &[RuinedPortalSetup] = &[RuinedPortalSetup {
    placement: VerticalPlacement::OnOceanFloor,
    air_pocket_probability: 0.0,
    mossiness: 0.5,
    overgrown: false,
    vines: true,
    can_be_cold: false,
    replace_with_blackstone: false,
    weight: 1.0,
}];
const MOUNTAIN_SETUPS: &[RuinedPortalSetup] = &[
    RuinedPortalSetup {
        placement: VerticalPlacement::InMountain,
        air_pocket_probability: 1.0,
        mossiness: 0.2,
        overgrown: false,
        vines: false,
        can_be_cold: true,
        replace_with_blackstone: false,
        weight: 0.5,
    },
    RuinedPortalSetup {
        placement: VerticalPlacement::OnLandSurface,
        air_pocket_probability: 0.5,
        mossiness: 0.2,
        overgrown: false,
        vines: false,
        can_be_cold: true,
        replace_with_blackstone: false,
        weight: 0.5,
    },
];
const OCEAN_SETUPS: &[RuinedPortalSetup] = &[RuinedPortalSetup {
    placement: VerticalPlacement::OnOceanFloor,
    air_pocket_probability: 0.0,
    mossiness: 0.8,
    overgrown: false,
    vines: false,
    can_be_cold: true,
    replace_with_blackstone: false,
    weight: 1.0,
}];
const NETHER_SETUPS: &[RuinedPortalSetup] = &[RuinedPortalSetup {
    placement: VerticalPlacement::InNether,
    air_pocket_probability: 0.5,
    mossiness: 0.0,
    overgrown: false,
    vines: false,
    can_be_cold: false,
    replace_with_blackstone: true,
    weight: 1.0,
}];
const STANDARD_SETUPS: &[RuinedPortalSetup] = &[
    RuinedPortalSetup {
        placement: VerticalPlacement::Underground,
        air_pocket_probability: 1.0,
        mossiness: 0.2,
        overgrown: false,
        vines: false,
        can_be_cold: true,
        replace_with_blackstone: false,
        weight: 0.5,
    },
    RuinedPortalSetup {
        placement: VerticalPlacement::OnLandSurface,
        air_pocket_probability: 0.5,
        mossiness: 0.2,
        overgrown: false,
        vines: false,
        can_be_cold: true,
        replace_with_blackstone: false,
        weight: 0.5,
    },
];

/// `data/minecraft/worldgen/structure/ruined_portal*.json`, in file order: the weighted
/// draw in `RuinedPortalStructure.findGenerationPoint` walks the list as written.
const fn setups_for(variant: StructureKeys) -> &'static [RuinedPortalSetup] {
    match variant {
        StructureKeys::RuinedPortalDesert => DESERT_SETUPS,
        StructureKeys::RuinedPortalJungle => JUNGLE_SETUPS,
        StructureKeys::RuinedPortalSwamp => SWAMP_SETUPS,
        StructureKeys::RuinedPortalMountain => MOUNTAIN_SETUPS,
        StructureKeys::RuinedPortalOcean => OCEAN_SETUPS,
        StructureKeys::RuinedPortalNether => NETHER_SETUPS,
        // `minecraft:ruined_portal`
        _ => STANDARD_SETUPS,
    }
}

/// Vanilla `RuinedPortalStructure.sample`: a probability of exactly 0 or 1 is decided
/// without touching the random.
fn sample(random: &mut RandomGenerator, limit: f32) -> bool {
    if limit == 0.0 {
        false
    } else if limit == 1.0 {
        true
    } else {
        random.next_f32() < limit
    }
}

/// `Mth.randomBetweenInclusive`.
fn random_between_inclusive(random: &mut RandomGenerator, min: i32, max: i32) -> i32 {
    random.next_bounded_i32(max - min + 1) + min
}

/// `RuinedPortalStructure.getRandomWithinInterval`.
fn random_within_interval(random: &mut RandomGenerator, min_preferred: i32, max: i32) -> i32 {
    if min_preferred < max {
        random_between_inclusive(random, min_preferred, max)
    } else {
        max
    }
}

/// `RuinedPortalStructure.findSuitableY`.
#[expect(clippy::too_many_arguments)]
fn find_suitable_y(
    random: &mut RandomGenerator,
    sampler: Option<&mut (dyn HeightSampler + '_)>,
    placement: VerticalPlacement,
    air_pocket: bool,
    surface_y_at_center: i32,
    y_span: i32,
    bounding_box: &BlockBox,
    world_min_y: i32,
) -> i32 {
    let min_y = world_min_y + 15;
    let new_y = match placement {
        VerticalPlacement::InNether => {
            if air_pocket {
                random_between_inclusive(random, 32, 100)
            } else if random.next_f32() < 0.5 {
                random_between_inclusive(random, 27, 29)
            } else {
                random_between_inclusive(random, 29, 100)
            }
        }
        VerticalPlacement::InMountain => {
            random_within_interval(random, 70, surface_y_at_center - y_span)
        }
        VerticalPlacement::Underground => {
            random_within_interval(random, min_y, surface_y_at_center - y_span)
        }
        VerticalPlacement::PartlyBuried => {
            surface_y_at_center - y_span + random_between_inclusive(random, 2, 8)
        }
        VerticalPlacement::OnLandSurface | VerticalPlacement::OnOceanFloor => surface_y_at_center,
    };

    let Some(sampler) = sampler else {
        return new_y;
    };

    let corners = [
        (bounding_box.min.x, bounding_box.min.z),
        (bounding_box.max.x, bounding_box.min.z),
        (bounding_box.min.x, bounding_box.max.z),
        (bounding_box.max.x, bounding_box.max.z),
    ];
    let ocean_floor = placement == VerticalPlacement::OnOceanFloor;

    let mut projected_y = new_y;
    while projected_y > min_y {
        let mut corners_on_solid_ground = 0;
        for (x, z) in corners {
            if sampler.column_is_opaque(x, z, projected_y, ocean_floor) {
                corners_on_solid_ground += 1;
                if corners_on_solid_ground == 3 {
                    return projected_y;
                }
            }
        }
        projected_y -= 1;
    }
    projected_y
}

pub struct RuinedPortalGenerator {
    pub variant: StructureKeys,
}

impl StructureGenerator for RuinedPortalGenerator {
    /// Vanilla `RuinedPortalStructure.findGenerationPoint`:
    ///
    /// ```java
    /// // (weighted setup draw only when setups.size() > 1)
    /// boolean airPocket = sample(random, setup.airPocketProbability());
    /// if (random.nextFloat() < 0.05F) templateLocation = GIANT_PORTALS[random.nextInt(3)];
    /// else                            templateLocation = PORTALS[random.nextInt(10)];
    /// Rotation rotation = Util.getRandom(Rotation.values(), random);
    /// Mirror mirror = random.nextFloat() < 0.5F ? Mirror.NONE : Mirror.FRONT_BACK;
    /// BlockPos pivot = new BlockPos(template.getSize().getX() / 2, 0, template.getSize().getZ() / 2);
    /// BlockPos basePosition = context.chunkPos().getWorldPosition();
    /// BoundingBox boundingBox = template.getBoundingBox(basePosition, rotation, pivot, mirror);
    /// BlockPos center = boundingBox.getCenter();
    /// int surfaceY = chunkGenerator.getBaseHeight(center.getX(), center.getZ(), ...) - 1;
    /// int projectedY = findSuitableY(random, ..., boundingBox.getYSpan(), boundingBox, ...);
    /// BlockPos origin = new BlockPos(basePosition.getX(), projectedY, basePosition.getZ());
    /// ```
    ///
    /// The piece sits at the chunk's *corner*, the pivot is only the rotation pivot, and the
    /// surface is sampled at the centre of the rotated bounding box, not at the chunk centre.
    fn get_structure_position(
        &self,
        context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        let StructureGeneratorContext {
            chunk_x,
            chunk_z,
            mut random,
            sea_level,
            min_y,
            mut height_sampler,
            ..
        } = context;

        let setups = setups_for(self.variant);
        let setup = if setups.len() > 1 {
            let total: f32 = setups.iter().map(|s| s.weight).sum();
            let mut pick = random.next_f32();
            let mut chosen = None;
            for candidate in setups {
                pick -= candidate.weight / total;
                if pick < 0.0 {
                    chosen = Some(candidate);
                    break;
                }
            }
            *chosen?
        } else {
            *setups.first()?
        };

        let air_pocket = sample(&mut random, setup.air_pocket_probability);
        let template_name = if random.next_f32() < 0.05 {
            GIANT_PORTALS[random.next_bounded_i32(GIANT_PORTALS.len() as i32) as usize]
        } else {
            PORTALS[random.next_bounded_i32(PORTALS.len() as i32) as usize]
        };
        let template = get_template(template_name)?;

        let rotation = Rotation::from_index(random.next_bounded_i32(4) as u8);
        let mirror = if random.next_f32() < 0.5 {
            Mirror::None
        } else {
            Mirror::FrontBack
        };

        let vertical_placement = setup.placement;
        let properties = RuinedPortalProperties {
            // `setup.canBeCold() && isCold(origin, biome, seaLevel)`; the biome test draws
            // nothing, so getting it wrong cannot shift the random stream.
            cold: setup.can_be_cold,
            mossiness: setup.mossiness,
            air_pocket,
            overgrown: setup.overgrown,
            vines: setup.vines,
            replace_with_blackstone: setup.replace_with_blackstone,
        };

        let pivot = Vector3::new(template.size.x / 2, 0, template.size.z / 2);
        let base_x = chunk_x * 16;
        let base_z = chunk_z * 16;
        let base_settings = make_settings(mirror, rotation, vertical_placement, pivot, &properties);
        let bounding_box =
            template.get_bounding_box(&base_settings, Vector3::new(base_x, 0, base_z));
        let bb_center = bounding_box.center();
        let center_x = bb_center.x;
        let center_z = bb_center.z;
        let y_span = bounding_box.max.y - bounding_box.min.y + 1;

        let ocean_floor = vertical_placement == VerticalPlacement::OnOceanFloor;
        let surface_y = height_sampler.as_deref_mut().map_or(sea_level, |sampler| {
            if ocean_floor {
                sampler.estimate_ocean_floor_height(center_x, center_z)
            } else {
                sampler.estimate_height(center_x, center_z)
            }
        }) - 1;

        let projected_y = find_suitable_y(
            &mut random,
            height_sampler,
            vertical_placement,
            air_pocket,
            surface_y,
            y_span,
            &bounding_box,
            min_y,
        );

        let template_position = Vector3::new(base_x, projected_y, base_z);

        let piece = RuinedPortalPiece::new(
            template,
            template_name.to_string(),
            template_position,
            vertical_placement,
            properties,
            rotation,
            mirror,
            pivot,
        );

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(piece));

        Some(StructurePosition {
            start_pos: BlockPos::new(base_x, projected_y, base_z),
            collector: Arc::new(collector.into()),
        })
    }
}

pub struct RuinedPortalPiece {
    pub piece: StructurePiece,
    pub template: Arc<StructureTemplate>,
    pub template_name: String,
    pub place_settings: StructurePlaceSettings,
    pub template_position: Vector3<i32>,
    pub vertical_placement: VerticalPlacement,
    pub properties: RuinedPortalProperties,
}

impl RuinedPortalPiece {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        template: Arc<StructureTemplate>,
        template_name: String,
        template_position: Vector3<i32>,
        vertical_placement: VerticalPlacement,
        properties: RuinedPortalProperties,
        rotation: Rotation,
        mirror: Mirror,
        pivot: Vector3<i32>,
    ) -> Self {
        let place_settings =
            make_settings(mirror, rotation, vertical_placement, pivot, &properties);
        let bounding_box = template.get_bounding_box(&place_settings, template_position);

        Self {
            piece: StructurePiece::new(StructurePieceType::RuinedPortal, bounding_box, 0),
            template,
            template_name,
            place_settings,
            template_position,
            vertical_placement,
            properties,
        }
    }

    fn place_blocks(
        &self,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        chunk_box: &BlockBox,
    ) {
        let rotation = self.place_settings.get_rotation();
        let mirror = self.place_settings.get_mirror();
        let pivot = self.place_settings.get_rotation_pivot();

        let mut context_rng = LegacyRand::from_seed(hash_block_pos(
            self.template_position.x,
            self.template_position.y,
            self.template_position.z,
        ) as u64);
        let mut context = ProcessorContext::new(
            self.template_position,
            self.place_settings.get_processors(),
            &mut context_rng,
        );

        for block in &self.template.blocks {
            let palette_entry = &self.template.palette[block.state as usize];

            let mut block_entity_nbt = block.nbt.clone();
            let placed_entry = palette_entry.clone();

            let Some(mut state) = BlockStateResolver::resolve(&placed_entry, rotation, mirror)
            else {
                continue;
            };

            let local_pos =
                StructureTemplate::transform_block_pos(block.pos, mirror, rotation, pivot);
            let world_pos = self.template_position + local_pos;

            if !chunk_box.contains_pos(&world_pos) {
                continue;
            }

            let mut processed_state = Some(state);
            let mut capped_idx = 0;
            for processor in self.place_settings.get_processors() {
                let Some(current_state) = processed_state else {
                    break;
                };
                processed_state = processor.process_with_context(
                    chunk,
                    world_pos,
                    current_state,
                    &mut block_entity_nbt,
                    &mut context,
                    &mut capped_idx,
                    &mut context_rng,
                );
            }

            let Some(final_state) = processed_state else {
                continue;
            };

            if chunk.get_block_state(&world_pos).to_block_id() == Block::WATER.id
                && let Some((_, waterlogged)) = placed_entry
                    .properties
                    .iter()
                    .find(|(name, _)| name == "waterlogged")
                && waterlogged == "true"
            {
                if let Some(waterlogged_state) =
                    BlockStateResolver::resolve(&placed_entry, rotation, mirror)
                {
                    state = waterlogged_state;
                } else {
                    state = final_state;
                }
            } else {
                state = final_state;
            }

            chunk.set_block_state(world_pos.x, world_pos.y, world_pos.z, state);

            let final_block = Block::from_id(state.id.to_block_id());
            let block_entity_id =
                crate::generation::structure::template::get_block_entity_id(final_block.name);
            if block_entity_nbt.is_some() || block_entity_id.is_some() {
                let fallback_id = block_entity_id.unwrap_or(final_block.name);
                let mut placed_nbt = pumpkin_nbt::compound::NbtCompound::new();

                placed_nbt.put_string("id", fallback_id.to_string());
                placed_nbt.put_int("x", world_pos.x);
                placed_nbt.put_int("y", world_pos.y);
                placed_nbt.put_int("z", world_pos.z);

                if let Some(template_nbt) = &block_entity_nbt {
                    for (key, value) in &template_nbt.child_tags {
                        if key.as_ref() != "x"
                            && key.as_ref() != "y"
                            && key.as_ref() != "z"
                            && key.as_ref() != "id"
                        {
                            placed_nbt.child_tags.insert(key.clone(), value.clone());
                        }
                    }
                }

                // `StructureTemplate.placeInWorld`, inside the placement loop:
                //
                //     if (blockInfo.nbt != null) {
                //         BlockEntity blockEntity = level.getBlockEntity(blockPos);
                //         if (blockEntity != null && blockEntity instanceof RandomizableContainer) {
                //             blockInfo.nbt.putLong("LootTableSeed", random.nextLong());
                //         }
                //     }
                //
                // The seed comes off the *feature* random, so the chest every ruined portal
                // ships costs the piece one `nextLong()` before `spreadNetherrack` runs.
                if block_entity_nbt.is_some() && placed_nbt.get_string("LootTable").is_some() {
                    placed_nbt.put_long("LootTableSeed", random.next_i64());
                }

                chunk.add_block_entity(placed_nbt);
            }
        }
    }

    fn spread_netherrack(&self, random: &mut RandomGenerator, cache: &mut dyn GenerationCache) {
        let follow_ground_surface = self.vertical_placement == VerticalPlacement::OnLandSurface
            || self.vertical_placement == VerticalPlacement::OnOceanFloor;
        let bb = self.piece.bounding_box;
        let center = bb.center();
        let center_x = center.x;
        let center_z = center.z;

        let max_distance = NETHERRACK_PROBABILITY_BY_DISTANCE.len() as i32;
        let x_span = bb.max.x - bb.min.x + 1;
        let z_span = bb.max.z - bb.min.z + 1;
        // Vanilla `(this.boundingBox.getXSpan() + this.boundingBox.getZSpan()) / 2`;
        // both spans are positive, so the midpoint is the same value.
        let average_width = i32::midpoint(x_span, z_span);
        let max_adj = (8 - average_width / 2).max(1);
        let distance_adjustment = random.next_bounded_i32(max_adj);

        let heightmap = self.vertical_placement.get_heightmap_type();

        for x in (center_x - max_distance)..=(center_x + max_distance) {
            for z in (center_z - max_distance)..=(center_z + max_distance) {
                let distance = (x - center_x).abs() + (z - center_z).abs();
                let adjusted_distance = (distance + distance_adjustment).max(0);
                if adjusted_distance < max_distance {
                    let prob = NETHERRACK_PROBABILITY_BY_DISTANCE[adjusted_distance as usize];
                    if random.next_f64() < prob as f64 {
                        let surface_y = cache.get_top_y(&heightmap, x, z) - 1;
                        let y = if follow_ground_surface {
                            surface_y
                        } else {
                            bb.min.y.min(surface_y)
                        };
                        let pos = Vector3::new(x, y, z);
                        if (y - bb.min.y).abs() <= 3
                            && self.can_block_be_replaced_by_netherrack_or_magma(cache, pos)
                        {
                            self.place_netherrack_or_magma(random, cache, pos);
                            if self.properties.overgrown {
                                Self::maybe_add_leaves_above(random, cache, pos);
                            }
                            self.add_netherrack_drip_column(
                                random,
                                cache,
                                pos + Vector3::new(0, -1, 0),
                            );
                        }
                    }
                }
            }
        }
    }

    fn can_block_be_replaced_by_netherrack_or_magma(
        &self,
        cache: &dyn GenerationCache,
        pos: Vector3<i32>,
    ) -> bool {
        let state = GenerationCache::get_block_state(cache, &pos);
        let block_id = state.to_block_id();
        block_id != Block::AIR.id
            && block_id != Block::OBSIDIAN.id
            && block_id != Block::BEDROCK.id
            && (self.vertical_placement == VerticalPlacement::InNether
                || block_id != Block::LAVA.id)
    }

    fn place_netherrack_or_magma(
        &self,
        random: &mut RandomGenerator,
        cache: &mut dyn GenerationCache,
        pos: Vector3<i32>,
    ) {
        if !self.properties.cold && random.next_f32() < 0.07 {
            cache.set_block_state(&pos, Block::MAGMA_BLOCK.default_state);
        } else {
            cache.set_block_state(&pos, Block::NETHERRACK.default_state);
        }
    }

    fn add_netherrack_drip_columns_below_portal(
        &self,
        random: &mut RandomGenerator,
        cache: &mut dyn GenerationCache,
    ) {
        let bb = self.piece.bounding_box;
        for x in (bb.min.x + 1)..bb.max.x {
            for z in (bb.min.z + 1)..bb.max.z {
                let pos = Vector3::new(x, bb.min.y, z);
                if GenerationCache::get_block_state(cache, &pos).to_block_id()
                    == Block::NETHERRACK.id
                {
                    self.add_netherrack_drip_column(
                        random,
                        cache,
                        Vector3::new(x, bb.min.y - 1, z),
                    );
                }
            }
        }
    }

    fn add_netherrack_drip_column(
        &self,
        random: &mut RandomGenerator,
        cache: &mut dyn GenerationCache,
        start_pos: Vector3<i32>,
    ) {
        let mut cur = start_pos;
        self.place_netherrack_or_magma(random, cache, cur);
        let mut remaining = 8;
        while remaining > 0 && random.next_f32() < 0.5 {
            cur.y -= 1;
            remaining -= 1;
            self.place_netherrack_or_magma(random, cache, cur);
        }
    }

    fn maybe_add_vines(
        random: &mut RandomGenerator,
        cache: &mut dyn GenerationCache,
        pos: Vector3<i32>,
    ) {
        let state = GenerationCache::get_block_state(cache, &pos);
        let block_id = state.to_block_id();
        if block_id != Block::AIR.id && block_id != Block::VINE.id {
            let dir_idx = random.next_bounded_i32(4);
            let (dir_offset, vine_face) = match dir_idx {
                0 => (Vector3::new(0, 0, -1), "south"),
                1 => (Vector3::new(0, 0, 1), "north"),
                2 => (Vector3::new(-1, 0, 0), "east"),
                _ => (Vector3::new(1, 0, 0), "west"),
            };
            let neighbor_pos = pos + dir_offset;
            if GenerationCache::get_block_state(cache, &neighbor_pos).to_block_id() == Block::AIR.id
            {
                let entry = PaletteEntry::with_properties(
                    "minecraft:vine".to_string(),
                    vec![(vine_face.to_string(), "true".to_string())],
                );
                if let Some(vine_state) =
                    BlockStateResolver::resolve(&entry, Rotation::None, Mirror::None)
                {
                    cache.set_block_state(&neighbor_pos, vine_state);
                }
            }
        }
    }

    fn maybe_add_leaves_above(
        random: &mut RandomGenerator,
        cache: &mut dyn GenerationCache,
        pos: Vector3<i32>,
    ) {
        if random.next_f32() < 0.5
            && GenerationCache::get_block_state(cache, &pos).to_block_id() == Block::NETHERRACK.id
        {
            let above = pos + Vector3::new(0, 1, 0);
            if GenerationCache::get_block_state(cache, &above).to_block_id() == Block::AIR.id {
                let entry = PaletteEntry::with_properties(
                    "minecraft:jungle_leaves".to_string(),
                    vec![("persistent".to_string(), "true".to_string())],
                );
                if let Some(leaves_state) =
                    BlockStateResolver::resolve(&entry, Rotation::None, Mirror::None)
                {
                    cache.set_block_state(&above, leaves_state);
                }
            }
        }
    }
}

impl StructurePieceBase for RuinedPortalPiece {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece
    }
    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece
    }
    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        _block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let bounding_box = self
            .template
            .get_bounding_box(&self.place_settings, self.template_position);
        let center = bounding_box.center();

        if chunk_box.contains_pos(&center) {
            let mut enlarged_box = *chunk_box;
            enlarged_box.encompass(&bounding_box);

            self.place_blocks(chunk, random, &enlarged_box);
        }
    }

    /// The tail of vanilla `RuinedPortalPiece.postProcess`. Everything after
    /// `super.postProcess` writes straight through the `WorldGenLevel`, with no chunk
    /// bounding box in the way:
    ///
    /// ```java
    /// chunkBB.encapsulate(boundingBox);
    /// super.postProcess(level, structureManager, generator, random, chunkBB, chunkPos, referencePos);
    /// this.spreadNetherrack(random, level);
    /// this.addNetherrackDripColumnsBelowPortal(random, level);
    /// ```
    ///
    /// so the netherrack mound reaches up to 14 blocks past the piece, into chunks the
    /// piece itself never touches.
    fn place_across_chunks(
        &mut self,
        cache: &mut dyn GenerationCache,
        random: &mut RandomGenerator,
        chunk_box: &BlockBox,
    ) {
        let bounding_box = self
            .template
            .get_bounding_box(&self.place_settings, self.template_position);
        let center = bounding_box.center();

        if chunk_box.contains_pos(&center) {
            self.spread_netherrack(random, cache);
            self.add_netherrack_drip_columns_below_portal(random, cache);

            if self.properties.vines || self.properties.overgrown {
                for x in self.piece.bounding_box.min.x..=self.piece.bounding_box.max.x {
                    for y in self.piece.bounding_box.min.y..=self.piece.bounding_box.max.y {
                        for z in self.piece.bounding_box.min.z..=self.piece.bounding_box.max.z {
                            let pos = Vector3::new(x, y, z);
                            if self.properties.vines {
                                Self::maybe_add_vines(random, cache, pos);
                            }
                            if self.properties.overgrown {
                                Self::maybe_add_leaves_above(random, cache, pos);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn make_settings(
    mirror: Mirror,
    rotation: Rotation,
    vertical_placement: VerticalPlacement,
    pivot: Vector3<i32>,
    properties: &RuinedPortalProperties,
) -> StructurePlaceSettings {
    let ignore_processor = if properties.air_pocket {
        StructureProcessor::BlockIgnore(vec![IgnoredBlock {
            block_id: Block::STRUCTURE_BLOCK.id,
            properties: None,
        }])
    } else {
        StructureProcessor::BlockIgnore(vec![
            IgnoredBlock {
                block_id: Block::STRUCTURE_BLOCK.id,
                properties: None,
            },
            IgnoredBlock {
                block_id: Block::AIR.id,
                properties: None,
            },
        ])
    };

    let mut rules = Vec::new();
    rules.push(ProcessorRule {
        position_predicate: PosRuleTest::AlwaysTrue,
        input_predicate: RuleTest::RandomBlockMatch {
            block: Block::GOLD_BLOCK.id,
            probability: 0.3,
        },
        location_predicate: RuleTest::AlwaysTrue,
        output_state: Block::AIR.default_state,
        block_entity_modifier: None,
    });

    let lava_rule = match vertical_placement {
        VerticalPlacement::OnOceanFloor => ProcessorRule {
            position_predicate: PosRuleTest::AlwaysTrue,
            input_predicate: RuleTest::BlockMatch(Block::LAVA.id),
            location_predicate: RuleTest::AlwaysTrue,
            output_state: Block::MAGMA_BLOCK.default_state,
            block_entity_modifier: None,
        },
        _ => {
            if properties.cold {
                ProcessorRule {
                    position_predicate: PosRuleTest::AlwaysTrue,
                    input_predicate: RuleTest::BlockMatch(Block::LAVA.id),
                    location_predicate: RuleTest::AlwaysTrue,
                    output_state: Block::NETHERRACK.default_state,
                    block_entity_modifier: None,
                }
            } else {
                ProcessorRule {
                    position_predicate: PosRuleTest::AlwaysTrue,
                    input_predicate: RuleTest::RandomBlockMatch {
                        block: Block::LAVA.id,
                        probability: 0.2,
                    },
                    location_predicate: RuleTest::AlwaysTrue,
                    output_state: Block::MAGMA_BLOCK.default_state,
                    block_entity_modifier: None,
                }
            }
        }
    };
    rules.push(lava_rule);

    if !properties.cold {
        rules.push(ProcessorRule {
            position_predicate: PosRuleTest::AlwaysTrue,
            input_predicate: RuleTest::RandomBlockMatch {
                block: Block::NETHERRACK.id,
                probability: 0.07,
            },
            location_predicate: RuleTest::AlwaysTrue,
            output_state: Block::MAGMA_BLOCK.default_state,
            block_entity_modifier: None,
        });
    }

    let mut settings = StructurePlaceSettings::new()
        .set_rotation(rotation)
        .set_mirror(mirror)
        .set_rotation_pivot(pivot)
        .add_processor(ignore_processor)
        .add_processor(StructureProcessor::Rule(rules))
        .add_processor(StructureProcessor::BlockAge {
            mossiness: properties.mossiness,
        })
        .add_processor(StructureProcessor::ProtectedBlocks(
            "#minecraft:features_cannot_replace".to_string(),
        ))
        .add_processor(StructureProcessor::LavaSubmergedBlock);

    if properties.replace_with_blackstone {
        settings = settings.add_processor(StructureProcessor::BlackstoneReplace);
    }

    settings
}

#[cfg(test)]
mod tests {
    use super::{RuinedPortalGenerator, VerticalPlacement, setups_for};
    use crate::generation::structure::structures::{
        StructureGenerator, StructureGeneratorContext, create_chunk_random,
    };
    use pumpkin_data::structures::StructureKeys;
    use pumpkin_util::math::block_box::BlockBox;

    /// `RuinedPortalStructure.findGenerationPoint` anchors the piece at the chunk's corner:
    ///
    /// ```java
    /// BlockPos basePosition = context.chunkPos().getWorldPosition();
    /// ...
    /// BlockPos origin = new BlockPos(basePosition.getX(), projectedY, basePosition.getZ());
    /// ```
    ///
    /// `ChunkPos.getWorldPosition()` is `new BlockPos(getMinBlockX(), 0, getMinBlockZ())`, and
    /// the pivot only turns the template — it never shifts the origin. Pumpkin used to place
    /// the template at `chunkCentre - pivot`, which moved every ruined portal (and the
    /// netherrack mound around it) by up to a chunk.
    #[test]
    fn a_ruined_portal_is_anchored_at_the_chunk_corner() {
        for (chunk_x, chunk_z) in [(0, 15), (-3, 7), (12, -5)] {
            let generator = RuinedPortalGenerator {
                variant: StructureKeys::RuinedPortalDesert,
            };
            let context = StructureGeneratorContext {
                seed: 13579,
                chunk_x,
                chunk_z,
                random: create_chunk_random(13579, chunk_x, chunk_z),
                sea_level: 63,
                min_y: -64,
                height_sampler: None,
                structure_key: Some(StructureKeys::RuinedPortalDesert),
            };
            let position = generator
                .get_structure_position(context)
                .expect("the desert ruined portal always produces a stub");
            assert_eq!(position.start_pos.0.x, chunk_x * 16);
            assert_eq!(position.start_pos.0.z, chunk_z * 16);
        }
    }

    /// The desert variant ships a single `partly_buried` setup with `air_pocket_probability`
    /// 0, so `RuinedPortalStructure.sample` short-circuits and no weighted setup draw
    /// happens: the first value the structure random is asked for is the giant-portal float.
    #[test]
    fn the_desert_setup_matches_the_datapack() {
        let setups = setups_for(StructureKeys::RuinedPortalDesert);
        assert_eq!(setups.len(), 1);
        assert_eq!(setups[0].placement, VerticalPlacement::PartlyBuried);
        assert!((setups[0].air_pocket_probability - 0.0).abs() < f32::EPSILON);
        assert!((setups[0].mossiness - 0.0).abs() < f32::EPSILON);
        assert!(!setups[0].can_be_cold);
        // `minecraft:ruined_portal` and `..._mountain` are the two with a weighted choice.
        assert_eq!(setups_for(StructureKeys::RuinedPortal).len(), 2);
        assert_eq!(setups_for(StructureKeys::RuinedPortalMountain).len(), 2);
    }

    /// Vanilla `BoundingBox.getCenter()` uses the inclusive spans, so an even span lands one
    /// block past the midpoint of the corners:
    ///
    /// ```java
    /// new BlockPos(this.minX() + this.getXSpan() / 2, ..., this.minZ() + this.getZSpan() / 2)
    /// ```
    #[test]
    fn a_bounding_box_centre_uses_the_inclusive_span() {
        // The desert portal of chunk (0, 15): x 0..9 (span 10), z 240..246 (span 7).
        let box_ = BlockBox::new(0, 63, 240, 9, 72, 246);
        let center = box_.center();
        assert_eq!(center.x, 5, "0 + 10 / 2");
        assert_eq!(center.y, 68, "63 + 10 / 2");
        assert_eq!(center.z, 243, "240 + 7 / 2");
    }
}
