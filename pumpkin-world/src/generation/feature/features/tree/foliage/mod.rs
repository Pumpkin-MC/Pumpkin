use acacia::AcaciaFoliagePlacer;
use blob::BlobFoliagePlacer;
use bush::BushFoliagePlacer;
use cherry::CherryFoliagePlacer;
use dark_oak::DarkOakFoliagePlacer;
use fancy::LargeOakFoliagePlacer;
use jungle::JungleFoliagePlacer;
use mega_pine::MegaPineFoliagePlacer;
use pine::PineFoliagePlacer;
use pumpkin_data::BlockDirection;
use pumpkin_data::BlockState;
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};
use random_spread::RandomSpreadFoliagePlacer;

use spruce::SpruceFoliagePlacer;

use super::{TreeFeature, TreeNode};
use crate::generation::proto_chunk::GenerationCache;

pub mod acacia;
pub mod blob;
pub mod bush;
pub mod cherry;
pub mod dark_oak;
pub mod fancy;
pub mod jungle;
pub mod mega_pine;
pub mod pine;
pub mod random_spread;
pub mod spruce;

pub struct FoliagePlacer {
    pub radius: IntProvider,
    pub offset: IntProvider,
    pub r#type: FoliageType,
}

pub trait LeaveValidator {
    fn is_position_invalid(
        &self,
        random: &mut RandomGenerator,
        dx: i32,
        y: i32,
        dz: i32,
        radius: i32,
        giant_trunk: bool,
    ) -> bool {
        let x = if giant_trunk {
            dx.abs().min((dx - 1).abs())
        } else {
            dx.abs()
        };
        let z = if giant_trunk {
            dz.abs().min((dz - 1).abs())
        } else {
            dz.abs()
        };
        self.is_invalid_for_leaves(random, x, y, z, radius, giant_trunk)
    }

    fn is_invalid_for_leaves(
        &self,
        random: &mut RandomGenerator,
        dx: i32,
        y: i32,
        dz: i32,
        radius: i32,
        giant_trunk: bool,
    ) -> bool;
}

impl FoliagePlacer {
    #[expect(clippy::too_many_arguments)]
    pub fn generate_square<T: LeaveValidator, T2: GenerationCache>(
        validator: &T,
        chunk: &mut T2,
        random: &mut RandomGenerator,
        center_pos: BlockPos,
        radius: i32,
        y: i32,
        giant_trunk: bool,
        foliage_provider: &BlockState,
    ) {
        let i = i32::from(giant_trunk);

        for x in -radius..=(radius + i) {
            for z in -radius..=(radius + i) {
                if validator.is_position_invalid(random, x, y, z, radius, giant_trunk) {
                    continue;
                }
                let pos = BlockPos(center_pos.0.add(&Vector3::new(x, y, z)));
                Self::place_foliage_block(chunk, pos, foliage_provider);
            }
        }
    }

    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        node: &TreeNode,
        foliage_height: i32,
        radius: i32,
        foliage_provider: &BlockState,
    ) {
        let offset = self.offset.get(random);
        self.r#type.generate(
            chunk,
            random,
            node,
            foliage_height,
            radius,
            offset,
            foliage_provider,
        );
    }

    pub fn get_random_radius(&self, random: &mut RandomGenerator, base_height: i32) -> i32 {
        match &self.r#type {
            FoliageType::Pine(_) => PineFoliagePlacer::get_random_radius(self, random, base_height),
            _ => self.radius.get(random),
        }
    }

    pub fn place_foliage_block<T: GenerationCache>(
        chunk: &mut T,
        pos: BlockPos,
        block_state: &BlockState,
    ) -> bool {
        let block = GenerationCache::get_block_state(chunk, &pos.0);
        if !TreeFeature::can_replace(block.to_state(), block.to_block_id()) {
            return false;
        }
        chunk.set_block_state(&pos.0, block_state);
        true
    }

    pub fn is_set<T: GenerationCache>(
        chunk: &T,
        pos: BlockPos,
        foliage_provider: &BlockState,
    ) -> bool {
        GenerationCache::get_block_state(chunk, &pos.0).0 == foliage_provider.id
    }

    fn try_place_extension<T: GenerationCache>(
        chunk: &mut T,
        random: &mut RandomGenerator,
        chance: f32,
        log_pos: BlockPos,
        pos: BlockPos,
        foliage_provider: &BlockState,
    ) -> bool {
        if pos.manhattan_distance(log_pos) >= 7 || random.next_f32() > chance {
            false
        } else {
            Self::place_foliage_block(chunk, pos, foliage_provider)
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub fn generate_square_with_hanging_leaves<T: LeaveValidator, T2: GenerationCache>(
        validator: &T,
        chunk: &mut T2,
        random: &mut RandomGenerator,
        center_pos: BlockPos,
        radius: i32,
        y: i32,
        giant_trunk: bool,
        foliage_provider: &BlockState,
        hanging_leaves_chance: f32,
        hanging_leaves_extension_chance: f32,
    ) {
        Self::generate_square(
            validator,
            chunk,
            random,
            center_pos,
            radius,
            y,
            giant_trunk,
            foliage_provider,
        );

        let i = i32::from(giant_trunk);
        let log_pos = center_pos.down();

        let directions = [
            BlockDirection::North,
            BlockDirection::South,
            BlockDirection::East,
            BlockDirection::West,
        ];

        for along_edge in directions {
            let to_edge = along_edge.rotate_clockwise();

            let offset_to_edge = if to_edge.positive() {
                radius + i
            } else {
                radius
            };

            let mut pos = center_pos
                .add(0, y - 1, 0)
                .offset_dir(to_edge.to_offset(), offset_to_edge)
                .offset_dir(along_edge.to_offset(), -radius);

            for _ in -radius..(radius + i) {
                let leaves_above = Self::is_set(chunk, pos.up(), foliage_provider);
                if leaves_above
                    && Self::try_place_extension(
                        chunk,
                        random,
                        hanging_leaves_chance,
                        log_pos,
                        pos,
                        foliage_provider,
                    )
                {
                    Self::try_place_extension(
                        chunk,
                        random,
                        hanging_leaves_extension_chance,
                        log_pos,
                        pos.down(),
                        foliage_provider,
                    );
                }
                pos = pos.offset_dir(along_edge.to_offset(), 1);
            }
        }
    }
}

pub enum FoliageType {
    Blob(BlobFoliagePlacer),
    Spruce(SpruceFoliagePlacer),
    Pine(PineFoliagePlacer),
    Acacia(AcaciaFoliagePlacer),
    Bush(BushFoliagePlacer),
    Fancy(LargeOakFoliagePlacer),
    Jungle(JungleFoliagePlacer),
    MegaPine(MegaPineFoliagePlacer),
    DarkOak(DarkOakFoliagePlacer),
    RandomSpread(RandomSpreadFoliagePlacer),
    Cherry(CherryFoliagePlacer),
}

impl FoliageType {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        node: &TreeNode,
        foliage_height: i32,
        radius: i32,
        offset: i32,
        foliage_provider: &BlockState,
    ) {
        match self {
            Self::Blob(blob) => blob.generate(
                chunk,
                random,
                node,
                foliage_height,
                radius,
                offset,
                foliage_provider,
            ),
            Self::Spruce(spruce) => spruce.generate(
                chunk,
                random,
                node,
                foliage_height,
                radius,
                offset,
                foliage_provider,
            ),
            Self::Pine(pine) => pine.generate(
                chunk,
                random,
                node,
                foliage_height,
                radius,
                offset,
                foliage_provider,
            ),
            Self::Acacia(acacia) => acacia.generate(
                chunk,
                random,
                node,
                foliage_height,
                radius,
                offset,
                foliage_provider,
            ),
            Self::Bush(bush) => bush.generate(
                chunk,
                random,
                node,
                foliage_height,
                radius,
                offset,
                foliage_provider,
            ),
            Self::Fancy(fancy) => fancy.generate(
                chunk,
                random,
                node,
                foliage_height,
                radius,
                offset,
                foliage_provider,
            ),
            Self::Jungle(jungle) => jungle.generate(
                chunk,
                random,
                node,
                foliage_height,
                radius,
                offset,
                foliage_provider,
            ),
            Self::MegaPine(mega_pine) => mega_pine.generate(
                chunk,
                random,
                node,
                foliage_height,
                radius,
                offset,
                foliage_provider,
            ),
            Self::DarkOak(dark_oak) => dark_oak.generate(
                chunk,
                random,
                node,
                foliage_height,
                radius,
                offset,
                foliage_provider,
            ),
            Self::RandomSpread(random_spread) => random_spread.generate(
                chunk,
                random,
                node,
                foliage_height,
                radius,
                offset,
                foliage_provider,
            ),
            Self::Cherry(cherry) => cherry.generate(
                chunk,
                random,
                node,
                foliage_height,
                radius,
                offset,
                foliage_provider,
            ),
        }
    }

    pub fn get_random_height(&self, random: &mut RandomGenerator, trunk_height: i32) -> i32 {
        match self {
            Self::Blob(blob) => blob.get_random_height(random),
            Self::Spruce(spruce) => spruce.get_random_height(random, trunk_height),
            Self::Pine(pine) => pine.get_random_height(random, trunk_height),
            Self::Acacia(acacia) => acacia.get_random_height(random),
            Self::Bush(bush) => bush.get_random_height(random),
            Self::Fancy(fancy) => fancy.get_random_height(random),
            Self::Jungle(jungle) => jungle.get_random_height(random, trunk_height),
            Self::MegaPine(mega_pine) => mega_pine.get_random_height(random, trunk_height),
            Self::DarkOak(dark_oak) => dark_oak.get_random_height(random),
            Self::RandomSpread(random_spread) => {
                random_spread.get_random_height(random, trunk_height)
            }
            Self::Cherry(cherry) => cherry.get_random_height(random),
        }
    }
}
