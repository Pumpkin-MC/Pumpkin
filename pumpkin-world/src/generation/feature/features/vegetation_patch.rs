use std::collections::HashSet;

use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3, vertical_surface_type::VerticalSurfaceType},
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::block_predicate::BlockPredicate;
use crate::generation::block_state_provider::BlockStateProvider;
use crate::generation::proto_chunk::GenerationCache;
use crate::world::BlockRegistryExt;

pub struct VegetationPatchFeature {
    pub replaceable: BlockPredicate,
    pub ground_state: BlockStateProvider,
    pub vegetation_feature: Box<crate::generation::feature::placed_features::PlacedFeature>,
    pub surface: VerticalSurfaceType,
    pub depth: pumpkin_util::math::int_provider::IntProvider,
    pub extra_bottom_block_chance: f32,
    pub vertical_range: i32,
    pub vegetation_chance: f32,
    pub xz_radius: pumpkin_util::math::int_provider::IntProvider,
    pub extra_edge_column_chance: f32,
}

impl VegetationPatchFeature {
    /// Returns the block direction that points "into" the surface (down for floor, up for ceiling).
    fn surface_direction(&self) -> pumpkin_data::BlockDirection {
        match self.surface {
            VerticalSurfaceType::Floor => pumpkin_data::BlockDirection::Down,
            VerticalSurfaceType::Ceiling => pumpkin_data::BlockDirection::Up,
        }
    }

    /// Shortcut for `self.surface_direction().to_offset()`.
    fn surface_offset(&self) -> Vector3<i32> {
        self.surface_direction().to_offset()
    }

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
        // Convert radius providers
        let x_radius = self.xz_radius.get(random) + 1;
        let z_radius = self.xz_radius.get(random) + 1;

        let surface = self.place_ground_patch(
            chunk,
            block_registry,
            random,
            pos,
            &self.replaceable,
            x_radius,
            z_radius,
        );

        self.distribute_vegetation(
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
    pub(crate) fn place_ground_patch<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn BlockRegistryExt,
        random: &mut RandomGenerator,
        origin: BlockPos,
        replaceable: &BlockPredicate,
        x_radius: i32,
        z_radius: i32,
    ) -> HashSet<BlockPos> {
        let mut surface = HashSet::new();

        // Determine "inwards" and "outwards" directions based on the surface
        let inwards = self.surface_direction();
        let outwards = inwards.opposite();

        for dx in -x_radius..=x_radius {
            let is_x_edge = dx == -x_radius || dx == x_radius;
            for dz in -z_radius..=z_radius {
                let is_z_edge = dz == -z_radius || dz == z_radius;
                let is_corner = is_x_edge && is_z_edge;
                let is_edge = is_x_edge || is_z_edge;
                let is_edge_but_not_corner = is_edge && !is_corner;

                if is_corner {
                    continue;
                }

                if is_edge_but_not_corner
                    && (self.extra_edge_column_chance == 0.0
                        || random.next_f32() > self.extra_edge_column_chance)
                {
                    continue;
                }

                let mut pos = origin.offset(Vector3::new(dx, 0, dz));

                // Move down until we hit non-air or exceed vertical range
                for _ in 0..self.vertical_range {
                    if !chunk.is_air(&pos.0) {
                        break;
                    }
                    pos = pos.offset(inwards.to_offset());
                }

                // Now back the other way until we reach air again
                for _ in 0..self.vertical_range {
                    if chunk.is_air(&pos.0) {
                        break;
                    }
                    pos = pos.offset(outwards.to_offset());
                }

                let below_pos = pos.offset(self.surface_offset());

                let below_state_raw = GenerationCache::get_block_state(chunk, &below_pos.0);
                if chunk.is_air(&pos.0)
                    && below_state_raw
                        .to_state()
                        .is_side_solid(self.surface_direction().opposite())
                {
                    // Compute depth variation
                    let mut depth = self.depth.get(random);
                    if self.extra_bottom_block_chance > 0.0
                        && random.next_f32() < self.extra_bottom_block_chance
                    {
                        depth += 1;
                    }

                    let ground_pos = below_pos;
                    if self.place_ground(
                        chunk,
                        block_registry,
                        replaceable,
                        random,
                        ground_pos,
                        depth,
                    ) {
                        surface.insert(ground_pos);
                    }
                }
            }
        }

        surface
    }

    fn place_ground<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn BlockRegistryExt,
        replaceable: &BlockPredicate,
        random: &mut RandomGenerator,
        mut below_pos: BlockPos,
        depth: i32,
    ) -> bool {
        for i in 0..depth {
            let state_to_place = self.ground_state.get(random, below_pos);
            let below_state_raw = GenerationCache::get_block_state(chunk, &below_pos.0);

            if state_to_place.id != below_state_raw.0 {
                if !replaceable.test(block_registry, chunk, &below_pos) {
                    return i != 0;
                }

                chunk.set_block_state(&below_pos.0, state_to_place);
                // Move in direction of surface
                below_pos = below_pos.offset(self.surface_offset());
            }
        }
        true
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn distribute_vegetation<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn BlockRegistryExt,
        random: &mut RandomGenerator,
        min_y: i8,
        height: u16,
        feature_name: &str,
        surface: HashSet<BlockPos>,
    ) {
        let opposite_dir = self.surface_direction().opposite();

        for surface_pos in surface {
            if self.vegetation_chance > 0.0 && random.next_f32() < self.vegetation_chance {
                let placement_pos = surface_pos.offset(opposite_dir.to_offset());
                let _ = self.vegetation_feature.generate(
                    chunk,
                    block_registry,
                    min_y,
                    height,
                    feature_name,
                    random,
                    placement_pos,
                );
            }
        }
    }
}
