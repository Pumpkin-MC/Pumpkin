#![allow(clippy::print_stdout, clippy::needless_pass_by_value)]

use crate::generation::{generator::WorldGenerator, get_world_gen, proto_chunk::ProtoChunk};
use pumpkin_data::dimension::Dimension;
use pumpkin_util::world_seed::Seed;

use super::surface_biomes;

pub(crate) struct TestBlockRegistry;
impl crate::world::WorldPortalExt for TestBlockRegistry {
    #[expect(clippy::too_many_lines, reason = "test block registry placement rules")]
    fn can_place_at(
        &self,
        block: &pumpkin_data::Block,
        state: &pumpkin_data::BlockState,
        block_accessor: &dyn crate::world::BlockAccessor,
        block_pos: &pumpkin_util::math::position::BlockPos,
    ) -> bool {
        use pumpkin_data::Block;
        use pumpkin_data::tag::Taggable;
        let block_below = block_accessor.get_block(&block_pos.down());
        if block == &Block::MANGROVE_PROPAGULE {
            let props =
                pumpkin_data::block_properties::MangrovePropaguleLikeProperties::from_state_id(
                    state.id,
                );
            if props.hanging {
                return block_accessor.get_block(&block_pos.up()).has_tag(
                    &pumpkin_data::tag::Block::MINECRAFT_SUPPORTS_HANGING_MANGROVE_PROPAGULE,
                );
            }
            return block_below
                .has_tag(&pumpkin_data::tag::Block::MINECRAFT_SUPPORTS_MANGROVE_PROPAGULE);
        }
        if block == &Block::TALL_GRASS
            || block == &Block::LARGE_FERN
            || block == &Block::SUNFLOWER
            || block == &Block::LILAC
            || block == &Block::ROSE_BUSH
            || block == &Block::PEONY
            || block == &Block::PITCHER_PLANT
        {
            let props =
                pumpkin_data::block_properties::TallSeagrassLikeProperties::from_state_id(state.id);
            if props.half == pumpkin_data::block_properties::DoubleBlockHalf::Upper {
                let (below_block, below_state) =
                    block_accessor.get_block_and_state(&block_pos.down());
                return below_block == block
                    && pumpkin_data::block_properties::TallSeagrassLikeProperties::from_state_id(
                        below_state.id,
                    )
                    .half
                        == pumpkin_data::block_properties::DoubleBlockHalf::Lower;
            }
            return block_below.has_tag(&pumpkin_data::tag::Block::MINECRAFT_SUPPORTS_VEGETATION);
        }
        if block == &Block::SHORT_GRASS
            || block == &Block::FERN
            || block == &Block::POPPY
            || block == &Block::DANDELION
            || block == &Block::BLUE_ORCHID
            || block == &Block::ALLIUM
            || block == &Block::AZURE_BLUET
            || block == &Block::RED_TULIP
            || block == &Block::ORANGE_TULIP
            || block == &Block::WHITE_TULIP
            || block == &Block::PINK_TULIP
            || block == &Block::OXEYE_DAISY
            || block == &Block::CORNFLOWER
            || block == &Block::LILY_OF_THE_VALLEY
            || block == &Block::WITHER_ROSE
            || block == &Block::SWEET_BERRY_BUSH
            || block == &Block::WILDFLOWERS
            || block == &Block::PINK_PETALS
            || block == &Block::BUSH
            || block == &Block::RED_SHRUB
            || block == &Block::FIREFLY_BUSH
            || block == &Block::CLOSED_EYEBLOSSOM
            || block == &Block::OPEN_EYEBLOSSOM
            || block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_SMALL_FLOWERS)
            || block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_SAPLINGS)
        {
            return block_below.has_tag(&pumpkin_data::tag::Block::MINECRAFT_SUPPORTS_VEGETATION);
        }
        if block == &Block::DEAD_BUSH
            || block == &Block::SHORT_DRY_GRASS
            || block == &Block::TALL_DRY_GRASS
        {
            return block_below
                .has_tag(&pumpkin_data::tag::Block::MINECRAFT_SUPPORTS_DRY_VEGETATION);
        }
        if block == &Block::AZALEA || block == &Block::FLOWERING_AZALEA {
            return block_below.has_tag(&pumpkin_data::tag::Block::MINECRAFT_SUPPORTS_AZALEA);
        }
        if block == &Block::LEAF_LITTER {
            return block_accessor
                .get_block_state(&block_pos.down())
                .is_side_solid(pumpkin_data::BlockDirection::Up);
        }
        if block == &Block::CACTUS_FLOWER {
            let below_state = block_accessor.get_block_state(&block_pos.down());
            return block_below
                .has_tag(&pumpkin_data::tag::Block::MINECRAFT_SUPPORT_OVERRIDE_CACTUS_FLOWER)
                || below_state.is_center_solid(pumpkin_data::BlockDirection::Up);
        }
        if block == &Block::CACTUS {
            for direction in pumpkin_data::BlockDirection::horizontal() {
                let (n_block, n_state) =
                    block_accessor.get_block_and_state(&block_pos.offset(direction.to_offset()));
                if n_state.is_solid() || n_block == &Block::LAVA {
                    return false;
                }
            }
            return (block_below == &Block::CACTUS
                || block_below.has_tag(&pumpkin_data::tag::Block::MINECRAFT_SUPPORTS_CACTUS))
                && !block_accessor.get_block_state(&block_pos.up()).is_liquid();
        }
        if block == &Block::SUGAR_CANE {
            if block_below == &Block::SUGAR_CANE {
                return true;
            }
            if block_below.has_tag(&pumpkin_data::tag::Block::MINECRAFT_SUPPORTS_SUGAR_CANE) {
                let below = block_pos.down();
                for direction in pumpkin_data::BlockDirection::horizontal() {
                    let n_pos = below.offset(direction.to_offset());
                    let n_id = block_accessor.get_block_state_id(&n_pos);
                    let n_block = n_id.to_block();
                    let is_water = n_block == &Block::WATER
                        || n_id.is_waterlogged()
                        || matches!(
                            n_block.id,
                            pumpkin_data::BlockId::KELP
                                | pumpkin_data::BlockId::KELP_PLANT
                                | pumpkin_data::BlockId::SEAGRASS
                                | pumpkin_data::BlockId::TALL_SEAGRASS
                                | pumpkin_data::BlockId::BUBBLE_COLUMN
                        );
                    if is_water
                        || n_block.has_tag(
                            &pumpkin_data::tag::Block::MINECRAFT_SUPPORTS_SUGAR_CANE_ADJACENTLY,
                        )
                    {
                        return true;
                    }
                }
            }
            return false;
        }
        if block == &Block::BAMBOO || block == &Block::BAMBOO_SAPLING {
            return block_below.has_tag(&pumpkin_data::tag::Block::MINECRAFT_SUPPORTS_BAMBOO);
        }
        if block == &Block::WARPED_FUNGUS {
            return block_below
                .has_tag(&pumpkin_data::tag::Block::MINECRAFT_SUPPORTS_WARPED_FUNGUS);
        }
        if block == &Block::CRIMSON_FUNGUS {
            return block_below
                .has_tag(&pumpkin_data::tag::Block::MINECRAFT_SUPPORTS_CRIMSON_FUNGUS);
        }
        if block == &Block::WARPED_ROOTS {
            return block_below.has_tag(&pumpkin_data::tag::Block::MINECRAFT_SUPPORTS_WARPED_ROOTS);
        }
        if block == &Block::CRIMSON_ROOTS {
            return block_below
                .has_tag(&pumpkin_data::tag::Block::MINECRAFT_SUPPORTS_CRIMSON_ROOTS);
        }
        if block == &Block::NETHER_SPROUTS {
            return block_below
                .has_tag(&pumpkin_data::tag::Block::MINECRAFT_SUPPORTS_NETHER_SPROUTS);
        }
        if block == &Block::NETHER_WART {
            return block_below.has_tag(&pumpkin_data::tag::Block::MINECRAFT_SUPPORTS_NETHER_WART);
        }
        if block == &Block::BROWN_MUSHROOM || block == &Block::RED_MUSHROOM {
            let below_state = block_accessor.get_block_state(&block_pos.down());
            return block_below.has_tag(
                &pumpkin_data::tag::Block::MINECRAFT_OVERRIDES_MUSHROOM_LIGHT_REQUIREMENT,
            ) || below_state.is_solid_render();
        }
        if block == &Block::SMALL_DRIPLEAF {
            if block_below.has_tag(&pumpkin_data::tag::Block::MINECRAFT_SUPPORTS_SMALL_DRIPLEAF) {
                return true;
            }
            let here_id = block_accessor.get_block_state_id(block_pos);
            let is_water_source =
                here_id == Block::WATER.default_state.id || here_id.is_waterlogged();
            return is_water_source
                && block_below.has_tag(&pumpkin_data::tag::Block::MINECRAFT_SUPPORTS_VEGETATION);
        }
        if block == &Block::BIG_DRIPLEAF {
            return block_below == &Block::BIG_DRIPLEAF
                || block_below == &Block::BIG_DRIPLEAF_STEM
                || block_below.has_tag(&pumpkin_data::tag::Block::MINECRAFT_SUPPORTS_BIG_DRIPLEAF);
        }
        if block == &Block::SPORE_BLOSSOM {
            let above_state = block_accessor.get_block_state(&block_pos.up());
            let here_id = block_accessor.get_block_state_id(block_pos);
            let is_water = here_id.is_waterlogged()
                || matches!(
                    pumpkin_data::fluid::Fluid::from_state_id(here_id),
                    Some(f) if *f == pumpkin_data::fluid::Fluid::WATER
                        || *f == pumpkin_data::fluid::Fluid::FLOWING_WATER
                );
            return above_state.is_center_solid(pumpkin_data::BlockDirection::Down) && !is_water;
        }
        if block == &Block::HANGING_ROOTS {
            let above_state = block_accessor.get_block_state(&block_pos.up());
            return above_state.is_side_solid(pumpkin_data::BlockDirection::Down);
        }
        if block == &Block::PALE_MOSS_CARPET || block == &Block::MOSS_CARPET {
            return !block_accessor.get_block_state(&block_pos.down()).is_air();
        }
        if block == &Block::SEAGRASS {
            let below_state = block_accessor.get_block_state(&block_pos.down());
            return below_state.is_side_solid(pumpkin_data::BlockDirection::Up)
                && !block_below
                    .has_tag(&pumpkin_data::tag::Block::MINECRAFT_CANNOT_SUPPORT_SEAGRASS);
        }
        if block == &Block::TALL_SEAGRASS {
            let props =
                pumpkin_data::block_properties::TallSeagrassLikeProperties::from_state_id(state.id);
            if props.half == pumpkin_data::block_properties::DoubleBlockHalf::Upper {
                let (below_block, below_state) =
                    block_accessor.get_block_and_state(&block_pos.down());
                return below_block == &Block::TALL_SEAGRASS
                    && pumpkin_data::block_properties::TallSeagrassLikeProperties::from_state_id(
                        below_state.id,
                    )
                    .half
                        == pumpkin_data::block_properties::DoubleBlockHalf::Lower;
            }
            let here_id = block_accessor.get_block_state_id(block_pos);
            let is_full_water = here_id == Block::WATER.default_state.id
                || here_id.is_waterlogged()
                || matches!(
                    here_id.to_block_id(),
                    pumpkin_data::BlockId::KELP
                        | pumpkin_data::BlockId::KELP_PLANT
                        | pumpkin_data::BlockId::SEAGRASS
                        | pumpkin_data::BlockId::TALL_SEAGRASS
                        | pumpkin_data::BlockId::BUBBLE_COLUMN
                );
            let below_state = block_accessor.get_block_state(&block_pos.down());
            return is_full_water
                && below_state.is_side_solid(pumpkin_data::BlockDirection::Up)
                && !block_below
                    .has_tag(&pumpkin_data::tag::Block::MINECRAFT_CANNOT_SUPPORT_SEAGRASS);
        }
        if block == &Block::COCOA {
            let props =
                pumpkin_data::block_properties::CocoaLikeProperties::from_state_id(state.id);
            let support_pos = block_pos.offset(props.facing.to_offset());
            let support_block = block_accessor.get_block(&support_pos);
            return support_block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_SUPPORTS_COCOA);
        }
        if block == &Block::SEA_PICKLE {
            let below_pos = block_pos.down();
            let below_state = block_accessor.get_block_state(&below_pos);
            return below_state.is_side_solid(pumpkin_data::BlockDirection::Up)
                || !below_state.collision_shapes.is_empty();
        }
        if block == &Block::LILY_PAD {
            let below_pos = block_pos.down();
            let below_id = block_accessor.get_block_state_id(&below_pos);
            let here_id = block_accessor.get_block_state_id(block_pos);
            let is_source_water = (block_below == &Block::WATER
                && below_id == Block::WATER.default_state.id)
                || below_id.is_waterlogged()
                || matches!(
                    block_below.id,
                    pumpkin_data::BlockId::KELP
                        | pumpkin_data::BlockId::KELP_PLANT
                        | pumpkin_data::BlockId::SEAGRASS
                        | pumpkin_data::BlockId::TALL_SEAGRASS
                        | pumpkin_data::BlockId::BUBBLE_COLUMN
                );
            let here_has_fluid = here_id.is_waterlogged()
                || pumpkin_data::fluid::Fluid::from_state_id(here_id).is_some()
                || matches!(
                    here_id.to_block_id(),
                    pumpkin_data::BlockId::KELP
                        | pumpkin_data::BlockId::KELP_PLANT
                        | pumpkin_data::BlockId::SEAGRASS
                        | pumpkin_data::BlockId::TALL_SEAGRASS
                        | pumpkin_data::BlockId::BUBBLE_COLUMN
                );
            return (is_source_water
                || block_below.has_tag(&pumpkin_data::tag::Block::MINECRAFT_SUPPORTS_LILY_PAD))
                && !here_has_fluid;
        }
        true
    }

    fn mirror(
        &self,
        block: &pumpkin_data::Block,
        state_id: pumpkin_data::BlockStateId,
        mirror: pumpkin_data::Mirror,
    ) -> &'static pumpkin_data::BlockState {
        block.mirror(state_id, mirror)
    }

    fn rotate(
        &self,
        block: &pumpkin_data::Block,
        state_id: pumpkin_data::BlockStateId,
        rotation: pumpkin_data::Rotation,
    ) -> &'static pumpkin_data::BlockState {
        block.rotate(state_id, rotation)
    }

    fn spawn_mobs_for_chunk_generation(
        &self,
        _cache: &mut dyn crate::generation::proto_chunk::GenerationCache,
        _biome: &'static pumpkin_data::biome::Biome,
        _chunk_x: i32,
        _chunk_z: i32,
    ) {
    }
}

pub(crate) struct TestGridCache {
    pub(crate) center_pos: (i32, i32),
    pub(crate) chunks: std::collections::HashMap<(i32, i32), ProtoChunk>,
}

impl crate::generation::height_limit::HeightLimitView for TestGridCache {
    fn height(&self) -> u16 {
        384
    }
    fn bottom_y(&self) -> i8 {
        -64
    }
}

impl crate::world::BlockAccessor for TestGridCache {
    fn get_block(
        &self,
        position: &pumpkin_util::math::position::BlockPos,
    ) -> &'static pumpkin_data::Block {
        self.get_block_state_id(position).to_block()
    }
    fn get_block_state(
        &self,
        position: &pumpkin_util::math::position::BlockPos,
    ) -> &'static pumpkin_data::BlockState {
        self.get_block_state_id(position).to_state()
    }
    fn get_block_state_id(
        &self,
        position: &pumpkin_util::math::position::BlockPos,
    ) -> pumpkin_data::BlockStateId {
        crate::generation::proto_chunk::GenerationCache::get_block_state(self, &position.0)
    }
    fn get_block_and_state(
        &self,
        position: &pumpkin_util::math::position::BlockPos,
    ) -> (
        &'static pumpkin_data::Block,
        &'static pumpkin_data::BlockState,
    ) {
        let id = self.get_block_state_id(position);
        pumpkin_data::BlockState::from_id_with_block(id)
    }
}

impl crate::generation::proto_chunk::GenerationCache for TestGridCache {
    fn get_center_chunk_mut(&mut self) -> &mut ProtoChunk {
        self.chunks.get_mut(&self.center_pos).unwrap()
    }
    fn get_center_chunk(&self) -> &ProtoChunk {
        self.chunks.get(&self.center_pos).unwrap()
    }
    fn get_chunk_mut(&mut self, chunk_x: i32, chunk_z: i32) -> Option<&mut ProtoChunk> {
        self.chunks.get_mut(&(chunk_x, chunk_z))
    }
    fn get_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&ProtoChunk> {
        self.chunks.get(&(chunk_x, chunk_z))
    }
    fn try_get_proto_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&ProtoChunk> {
        self.chunks.get(&(chunk_x, chunk_z))
    }
    fn get_block_state(
        &self,
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
    ) -> pumpkin_data::BlockStateId {
        let cx = pos.x >> 4;
        let cz = pos.z >> 4;
        self.chunks
            .get(&(cx, cz))
            .map_or(pumpkin_data::BlockStateId::AIR, |chunk| {
                chunk.get_block_state(pos)
            })
    }
    fn get_fluid_and_fluid_state(
        &self,
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
    ) -> (pumpkin_data::fluid::Fluid, pumpkin_data::fluid::FluidState) {
        use pumpkin_data::fluid::Fluid;
        let id = crate::generation::proto_chunk::GenerationCache::get_block_state(self, pos);
        let Some(fluid) = Fluid::from_state_id(id) else {
            let fluid = if id.is_waterlogged()
                || matches!(
                    id.to_block_id(),
                    pumpkin_data::BlockId::KELP
                        | pumpkin_data::BlockId::KELP_PLANT
                        | pumpkin_data::BlockId::SEAGRASS
                        | pumpkin_data::BlockId::TALL_SEAGRASS
                        | pumpkin_data::BlockId::BUBBLE_COLUMN
                ) {
                Fluid::WATER
            } else {
                Fluid::EMPTY
            };
            let state = fluid.states[0].clone();
            return (fluid, state);
        };
        let state = fluid.states[0].clone();
        (fluid.clone(), state)
    }
    fn set_block_state(
        &mut self,
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        block_state: &pumpkin_data::BlockState,
    ) {
        let cx = pos.x >> 4;
        let cz = pos.z >> 4;
        if (cx - self.center_pos.0).abs() > 1 || (cz - self.center_pos.1).abs() > 1 {
            return;
        }
        if let Some(chunk) = self.chunks.get_mut(&(cx, cz)) {
            chunk.set_block_state(pos.x, pos.y, pos.z, block_state);
        }
    }
    fn add_block_entity(
        &mut self,
        _pos: &pumpkin_util::math::vector3::Vector3<i32>,
        nbt: pumpkin_nbt::compound::NbtCompound,
    ) {
        self.chunks
            .get_mut(&self.center_pos)
            .unwrap()
            .add_block_entity(nbt);
    }
    fn top_motion_blocking_block_height_exclusive(&self, x: i32, z: i32) -> i32 {
        let cx = x >> 4;
        let cz = z >> 4;
        self.chunks.get(&(cx, cz)).map_or(-64, |chunk| {
            chunk.top_motion_blocking_block_height_exclusive(x, z)
        })
    }
    fn top_motion_blocking_block_no_leaves_height_exclusive(&self, x: i32, z: i32) -> i32 {
        let cx = x >> 4;
        let cz = z >> 4;
        self.chunks.get(&(cx, cz)).map_or(-64, |chunk| {
            chunk.top_motion_blocking_block_no_leaves_height_exclusive(x, z)
        })
    }
    fn get_top_y(&self, heightmap: &pumpkin_util::HeightMap, x: i32, z: i32) -> i32 {
        let cx = x >> 4;
        let cz = z >> 4;
        self.chunks
            .get(&(cx, cz))
            .map_or(-64, |chunk| chunk.get_top_y(heightmap, x, z))
    }
    fn top_block_height_exclusive(&self, x: i32, z: i32) -> i32 {
        let cx = x >> 4;
        let cz = z >> 4;
        self.chunks
            .get(&(cx, cz))
            .map_or(-64, |chunk| chunk.top_block_height_exclusive(x, z))
    }
    fn top_block_wg_height_exclusive(&self, x: i32, z: i32) -> i32 {
        let cx = x >> 4;
        let cz = z >> 4;
        self.chunks
            .get(&(cx, cz))
            .map_or(-64, |chunk| chunk.top_block_wg_height_exclusive(x, z))
    }
    fn ocean_floor_height_exclusive(&self, x: i32, z: i32) -> i32 {
        let cx = x >> 4;
        let cz = z >> 4;
        self.chunks
            .get(&(cx, cz))
            .map_or(-64, |chunk| chunk.ocean_floor_height_exclusive(x, z))
    }
    fn ocean_floor_wg_height_exclusive(&self, x: i32, z: i32) -> i32 {
        let cx = x >> 4;
        let cz = z >> 4;
        self.chunks
            .get(&(cx, cz))
            .map_or(-64, |chunk| chunk.ocean_floor_wg_height_exclusive(x, z))
    }
    fn is_air(&self, local_pos: &pumpkin_util::math::vector3::Vector3<i32>) -> bool {
        pumpkin_data::block_properties::is_air(
            crate::generation::proto_chunk::GenerationCache::get_block_state(self, local_pos),
        )
    }
    fn get_biome_for_terrain_gen(
        &self,
        x: i32,
        y: i32,
        z: i32,
    ) -> &'static pumpkin_data::biome::Biome {
        let biome_pos = self.get_center_chunk().get_terrain_gen_biome_pos(x, y, z);
        let cx = biome_pos.x >> 2;
        let cz = biome_pos.z >> 2;
        self.chunks.get(&(cx, cz)).map_or(
            pumpkin_data::biome::Biome::from_id(pumpkin_data::chunk::Biome::PLAINS.id).unwrap(),
            |chunk| chunk.get_biome(biome_pos.x, biome_pos.y, biome_pos.z),
        )
    }
    fn get_blending_data(
        &self,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Option<&crate::generation::blender::blending_data::BlendingData> {
        self.chunks
            .get(&(chunk_x, chunk_z))
            .and_then(|c| c.blending_data.as_ref())
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct ChunkStageSectionChecksums {
    pub x: i32,
    pub z: i32,
    pub stage: String,
    pub sections: Vec<u64>,
}

#[must_use]
pub fn compute_section_checksum(chunk: &ProtoChunk, section_index: usize) -> u64 {
    const FNV_PRIME: u64 = 0x0000_0100_0000_01B3;
    let section_bottom_y = chunk.bottom_y() as i32 + (section_index as i32 * 16);
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;

    for y in 0..16 {
        for z in 0..16 {
            for x in 0..16 {
                let actual = chunk.get_block_state_raw(x, y + section_bottom_y, z);
                hash ^= actual.as_u16() as u64;
                hash = hash.wrapping_mul(FNV_PRIME);
            }
        }
    }
    hash
}

pub fn assert_chunk_sections_match(chunk: &ProtoChunk, expected_hashes: &[u64], stage_name: &str) {
    let section_count = chunk.height() as usize / 16;
    for (sec_idx, &expected_hash) in expected_hashes.iter().enumerate().take(section_count) {
        let actual_hash = compute_section_checksum(chunk, sec_idx);
        if actual_hash != expected_hash {
            let section_y = chunk.bottom_y() as i32 + (sec_idx as i32 * 16);
            panic!(
                "[{stage_name}] Section #{sec_idx} (Y: {section_y}..{}) hash mismatch at chunk ({}, {})! Actual hash: {actual_hash:016x}, Expected: {expected_hash:016x}",
                section_y + 15,
                chunk.x,
                chunk.z
            );
        }
    }
}

#[derive(serde::Deserialize, Debug)]
struct StageEntry {
    hash: String,
    sections: Vec<String>,
}

#[derive(serde::Deserialize, Debug)]
struct ChunkStageDump {
    x: i32,
    z: i32,
    dimension: String,
    stages: std::collections::HashMap<String, StageEntry>,
}

fn compute_chunk_stage_hashes(chunk: &ProtoChunk) -> (String, Vec<String>) {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    let section_count = (chunk.height() as usize) / 16;
    let mut section_checksums = Vec::with_capacity(section_count);
    let fnv_prime: u64 = 0x00000100000001B3;

    for sec_idx in 0..section_count {
        let mut sec_hash: u64 = 0xcbf29ce484222325;
        let base_y = (sec_idx * 16) as i32;

        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    let state_id = chunk.get_block_state_raw(x, base_y + y, z).as_u16();
                    sec_hash = (sec_hash ^ (state_id as u64)).wrapping_mul(fnv_prime);

                    let state_u32 = state_id as u32;
                    hasher.update(state_u32.to_be_bytes());
                }
            }
        }
        section_checksums.push(sec_hash.to_string());
    }

    let finalize_bytes = hasher.finalize();
    let mut overall_hash = String::with_capacity(64);
    for byte in finalize_bytes {
        use std::fmt::Write;
        let _ = write!(overall_hash, "{byte:02x}");
    }
    (overall_hash, section_checksums)
}

fn assert_stage_hashes(
    stage_name: &str,
    cx: i32,
    cz: i32,
    actual_hash: &str,
    actual_sections: &[String],
    expected: &StageEntry,
) {
    if actual_hash == expected.hash {
        assert_eq!(actual_sections, expected.sections);
        println!("[{stage_name} ({cx}, {cz})] 1:1 MATCH (hash: {actual_hash})");
        return;
    }

    let mut mismatch_count = 0;
    for (sec_idx, (act, exp)) in actual_sections.iter().zip(&expected.sections).enumerate() {
        if act != exp {
            mismatch_count += 1;
            println!(
                "[{stage_name} ({cx}, {cz})] Section {sec_idx} mismatch! Actual: {act}, Expected: {exp}"
            );
        }
    }
    println!(
        "[{stage_name} ({cx}, {cz})] Hash mismatch: actual {actual_hash} vs expected {}. Deviant sections: {mismatch_count}/{}",
        expected.hash,
        expected.sections.len()
    );
    assert_eq!(actual_hash, expected.hash);
}

fn verify_stage_parity(seed: u64, stage_dumps: &[ChunkStageDump]) {
    let block_registry = TestBlockRegistry;

    for chunk_dump in stage_dumps {
        let dim = match chunk_dump.dimension.as_str() {
            "nether" => Dimension::THE_NETHER,
            "end" => Dimension::THE_END,
            _ => Dimension::OVERWORLD,
        };
        let seed = Seed(seed);
        let world_gen = get_world_gen(seed, dim.clone(), false, Vec::new(), String::new());
        let WorldGenerator::Noise(generator) = &*world_gen else {
            unreachable!()
        };

        let cx = chunk_dump.x;
        let cz = chunk_dump.z;

        // Pre-populate neighborhood around (cx, cz)
        // Note: vanilla requires structure starts in radius 8
        let mut chunks_3x3 = std::collections::HashMap::new();
        for nx in (cx - 8)..=(cx + 8) {
            for nz in (cz - 8)..=(cz + 8) {
                let mut nchunk = ProtoChunk::new(nx, nz, &world_gen);
                nchunk.step_to_biomes(generator);
                nchunk.set_structure_starts(generator);
                nchunk.set_structure_references(generator);
                chunks_3x3.insert((nx, nz), nchunk);
            }
        }

        // Prepare center chunk
        let center = chunks_3x3.get_mut(&(cx, cz)).unwrap();

        // Stage 1: Noise
        center.step_to_noise(generator);
        if let Some(expected) = chunk_dump.stages.get("minecraft:noise") {
            let (actual_hash, actual_sections) = compute_chunk_stage_hashes(center);
            assert_stage_hashes("noise", cx, cz, &actual_hash, &actual_sections, expected);
        }

        // Stage 2: Surface
        let center_surface_biomes = surface_biomes(&world_gen, cx, cz);
        center.step_to_surface(generator, &center_surface_biomes);
        if let Some(expected) = chunk_dump.stages.get("minecraft:surface") {
            let (actual_hash, actual_sections) = compute_chunk_stage_hashes(center);
            assert_stage_hashes("surface", cx, cz, &actual_hash, &actual_sections, expected);
        }

        // Stage 3: Carvers
        center.step_to_carvers(generator);
        if let Some(expected) = chunk_dump.stages.get("minecraft:carvers") {
            let (actual_hash, actual_sections) = compute_chunk_stage_hashes(center);
            assert_stage_hashes("carvers", cx, cz, &actual_hash, &actual_sections, expected);
        }

        // Stage 4: Features
        // Advance 3x3 neighbors to carvers for feature decoration context
        for (&(nx, nz), nchunk) in &mut chunks_3x3 {
            if (nx, nz) == (cx, cz) || (nx - cx).abs() > 1 || (nz - cz).abs() > 1 {
                continue;
            }
            nchunk.step_to_noise(generator);
            let n_surface_biomes = surface_biomes(&world_gen, nx, nz);
            nchunk.step_to_surface(generator, &n_surface_biomes);
            nchunk.step_to_carvers(generator);
        }

        let mut grid_cache = TestGridCache {
            center_pos: (cx, cz),
            chunks: chunks_3x3,
        };

        ProtoChunk::generate_features_and_structure(
            &mut grid_cache,
            &block_registry,
            &generator.random_config,
        );

        let center = grid_cache.chunks.get(&(cx, cz)).unwrap();
        if let Some(expected) = chunk_dump.stages.get("minecraft:features") {
            let (actual_hash, actual_sections) = compute_chunk_stage_hashes(center);
            assert_stage_hashes("features", cx, cz, &actual_hash, &actual_sections, expected);
        }
    }
}

#[test]
fn stage_parity_hashes() {
    let stages: Vec<ChunkStageDump> =
        pumpkin_util::read_data_from_file!("../../../../../assets/tests/stage_hashes.json");
    verify_stage_parity(0, &stages);
}

#[test]
fn nether_stage_parity_hashes() {
    let stages: Vec<ChunkStageDump> =
        pumpkin_util::read_data_from_file!("../../../../../assets/tests/nether_stage_hashes.json");
    verify_stage_parity(0, &stages);
}

#[test]
fn end_stage_parity_hashes() {
    let stages: Vec<ChunkStageDump> =
        pumpkin_util::read_data_from_file!("../../../../../assets/tests/end_stage_hashes.json");
    verify_stage_parity(0, &stages);
}
