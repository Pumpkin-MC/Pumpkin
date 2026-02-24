use std::collections::HashSet;

use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};

use crate::generation::proto_chunk::GenerationCache;
use crate::world::BlockRegistryExt;

use super::vegetation_patch::VegetationPatchFeature;

pub struct WaterloggedVegetationPatchFeature {
    pub base: VegetationPatchFeature,
}

impl WaterloggedVegetationPatchFeature {
    #[allow(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn BlockRegistryExt,
        min_y: i8,
        height: u16,
        feature_name: &str,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let x_radius = self.base.xz_radius.get(random) + 1;
        let z_radius = self.base.xz_radius.get(random) + 1;

        let surface = self.place_ground_patch(
            chunk,
            block_registry,
            random,
            pos,
            &self.base.replaceable,
            x_radius,
            z_radius,
        );

        self.base.distribute_vegetation(
            chunk,
            block_registry,
            random,
            min_y,
            height,
            feature_name,
            surface.clone(),
        );

        !surface.is_empty()
    }

    #[allow(clippy::too_many_arguments)]
    fn place_ground_patch<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn BlockRegistryExt,
        random: &mut RandomGenerator,
        origin: BlockPos,
        replaceable: &crate::generation::block_predicate::BlockPredicate,
        x_radius: i32,
        z_radius: i32,
    ) -> HashSet<BlockPos> {
        let surface = self.base.place_ground_patch(
            chunk,
            block_registry,
            random,
            origin,
            replaceable,
            x_radius,
            z_radius,
        );

        let mut water_surface = HashSet::new();
        let mut test_pos = BlockPos::ZERO;

        for &surface_pos in &surface {
            if !is_exposed(chunk, &surface, surface_pos, &mut test_pos) {
                water_surface.insert(surface_pos);
            }
        }

        for pos in &water_surface {
            chunk.set_block_state(&pos.0, pumpkin_data::Block::WATER.default_state);
        }

        water_surface
    }

    #[allow(clippy::too_many_arguments)]
    fn place_vegetation<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn BlockRegistryExt,
        min_y: i8,
        height: u16,
        feature_name: &str,
        random: &mut RandomGenerator,
        placement_pos: BlockPos,
    ) -> bool {
        if self.base.vegetation_feature.generate(
            chunk,
            block_registry,
            min_y,
            height,
            feature_name,
            random,
            placement_pos,
        ) {
            let placed_raw = GenerationCache::get_block_state(chunk, &placement_pos.0);
            let placed_state = placed_raw.to_state();
            if !placed_state.is_waterlogged() {
                let block = placed_raw.to_block();
                let mut props: Vec<(&str, &str)> = block
                    .properties(placed_raw.0)
                    .map(|p| p.to_props().iter().map(|(k, v)| (*k, *v)).collect())
                    .unwrap_or_default();
                props.push(("waterlogged", "true"));
                let new_state_id = block.from_properties(&props).to_state_id(block);
                chunk.set_block_state(
                    &placement_pos.0,
                    pumpkin_data::BlockState::from_id(new_state_id),
                );
            }
            true
        } else {
            false
        }
    }
}

fn is_exposed<T: GenerationCache>(
    chunk: &T,
    surface: &HashSet<BlockPos>,
    pos: BlockPos,
    test_pos: &mut BlockPos,
) -> bool {
    is_exposed_direction(
        chunk,
        surface,
        pos,
        test_pos,
        pumpkin_data::BlockDirection::North,
    ) || is_exposed_direction(
        chunk,
        surface,
        pos,
        test_pos,
        pumpkin_data::BlockDirection::East,
    ) || is_exposed_direction(
        chunk,
        surface,
        pos,
        test_pos,
        pumpkin_data::BlockDirection::South,
    ) || is_exposed_direction(
        chunk,
        surface,
        pos,
        test_pos,
        pumpkin_data::BlockDirection::West,
    ) || is_exposed_direction(
        chunk,
        surface,
        pos,
        test_pos,
        pumpkin_data::BlockDirection::Down,
    )
}

fn is_exposed_direction<T: GenerationCache>(
    chunk: &T,
    _surface: &HashSet<BlockPos>,
    pos: BlockPos,
    test_pos: &mut BlockPos,
    direction: pumpkin_data::BlockDirection,
) -> bool {
    *test_pos = pos.offset(direction.to_offset());
    !GenerationCache::get_block_state(chunk, &test_pos.0)
        .to_state()
        .is_side_solid(direction.opposite())
}
